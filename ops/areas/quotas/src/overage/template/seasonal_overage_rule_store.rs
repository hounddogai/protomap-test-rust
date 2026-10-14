use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal overage rules by code.
pub trait SeasonalOverageRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalOverageRule>;
    fn save(&self, item: &SeasonalOverageRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalOverageRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalOverageRuleStore {
    pub items: RefCell<BTreeMap<String, SeasonalOverageRule>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalOverageRuleStore for MemorySeasonalOverageRuleStore {
    fn find(&self, code: &str) -> Option<SeasonalOverageRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalOverageRule) {
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

    fn list(&self) -> Vec<SeasonalOverageRule> {
        let mut out: Vec<SeasonalOverageRule> = Vec::new();
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

pub fn new_memory_seasonal_overage_rule_store() -> Rc<dyn SeasonalOverageRuleStore> {
    Rc::new(MemorySeasonalOverageRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalOverageRuleStore {
    pub inner: Rc<dyn SeasonalOverageRuleStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalOverageRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalOverageRuleStore for CachedSeasonalOverageRuleStore {
    fn find(&self, code: &str) -> Option<SeasonalOverageRule> {
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

    fn save(&self, item: &SeasonalOverageRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalOverageRule> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_overage_rule_store(
    inner: Rc<dyn SeasonalOverageRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalOverageRuleStore> {
    Rc::new(CachedSeasonalOverageRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalOverageRule>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal overage rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalOverageRuleQuery {
    pub country: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalOverageRuleQuery {
    pub fn matches(&self, item: &SeasonalOverageRule) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
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

    pub fn apply(&self, items: &[SeasonalOverageRule]) -> Vec<SeasonalOverageRule> {
        let matched = collections::filter(items, |candidate: &SeasonalOverageRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_overage_rules(
    store: Rc<dyn SeasonalOverageRuleStore>,
    query: &SeasonalOverageRuleQuery,
) -> Vec<SeasonalOverageRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalOverageRuleStore> = new_memory_seasonal_overage_rule_store();
        let item = new_seasonal_overage_rule("code-458", "name-582");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
