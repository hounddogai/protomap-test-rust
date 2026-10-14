use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal failover rules by code.
pub trait SeasonalFailoverRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalFailoverRule>;
    fn save(&self, item: &SeasonalFailoverRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalFailoverRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalFailoverRuleStore {
    pub items: RefCell<BTreeMap<String, SeasonalFailoverRule>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalFailoverRuleStore for MemorySeasonalFailoverRuleStore {
    fn find(&self, code: &str) -> Option<SeasonalFailoverRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalFailoverRule) {
        if !self.items.borrow().contains_key(&item.code) {
            self.order.borrow_mut().push(item.code.clone());
        }
        self.items.borrow_mut().insert(item.code.clone(), item.clone());
    }

    fn remove(&self, code: &str) -> bool {
        if !self.items.borrow().contains_key(code) {
            return false;
        }
        self.items.borrow_mut().remove(code);
        let mut kept: Vec<String> = Vec::new();
        for existing in self.order.borrow().iter() {
            if existing.as_str() != code {
                kept.push(existing.clone());
            }
        }
        *self.order.borrow_mut() = kept;
        true
    }

    fn list(&self) -> Vec<SeasonalFailoverRule> {
        let mut out: Vec<SeasonalFailoverRule> = Vec::new();
        for key in self.order.borrow().iter() {
            let found = self.items.borrow().get(key).cloned();
            let Some(found) = found else {
                continue;
            };
            out.push(found);
        }
        out
    }
}

pub fn new_memory_seasonal_failover_rule_store() -> Rc<dyn SeasonalFailoverRuleStore> {
    Rc::new(MemorySeasonalFailoverRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalFailoverRuleStore {
    pub inner: Rc<dyn SeasonalFailoverRuleStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalFailoverRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalFailoverRuleStore for CachedSeasonalFailoverRuleStore {
    fn find(&self, code: &str) -> Option<SeasonalFailoverRule> {
        let hit = self.cache.borrow_mut().get(code);
        let Some(hit) = hit else {
            self.misses.set(self.misses.get() + 1);
            let Some(found) = self.inner.find(code) else {
                return None;
            };
            self.cache.borrow_mut().put(code, &found);
            return Some(found);
        };
        self.hits.set(self.hits.get() + 1);
        Some(hit)
    }

    fn save(&self, item: &SeasonalFailoverRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalFailoverRule> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_failover_rule_store(
    inner: Rc<dyn SeasonalFailoverRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalFailoverRuleStore> {
    Rc::new(CachedSeasonalFailoverRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalFailoverRule>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal failover rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFailoverRuleQuery {
    pub name: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalFailoverRuleQuery {
    pub fn matches(&self, item: &SeasonalFailoverRule) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.attempts < self.min_attempts {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalFailoverRule]) -> Vec<SeasonalFailoverRule> {
        let matched = collections::filter(items, |candidate: &SeasonalFailoverRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_failover_rules(
    store: Rc<dyn SeasonalFailoverRuleStore>,
    query: &SeasonalFailoverRuleQuery,
) -> Vec<SeasonalFailoverRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalFailoverRuleStore> = new_memory_seasonal_failover_rule_store();
        let item = new_seasonal_failover_rule("code-672", "name-475");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
