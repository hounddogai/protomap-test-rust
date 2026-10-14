use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient health check rules by code.
pub trait LenientHealthCheckRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientHealthCheckRule>;
    fn save(&self, item: &LenientHealthCheckRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientHealthCheckRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientHealthCheckRuleStore {
    pub items: RefCell<BTreeMap<String, LenientHealthCheckRule>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientHealthCheckRuleStore for MemoryLenientHealthCheckRuleStore {
    fn find(&self, code: &str) -> Option<LenientHealthCheckRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientHealthCheckRule) {
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

    fn list(&self) -> Vec<LenientHealthCheckRule> {
        let mut out: Vec<LenientHealthCheckRule> = Vec::new();
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

pub fn new_memory_lenient_health_check_rule_store() -> Rc<dyn LenientHealthCheckRuleStore> {
    Rc::new(MemoryLenientHealthCheckRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientHealthCheckRuleStore {
    pub inner: Rc<dyn LenientHealthCheckRuleStore>,
    pub cache: RefCell<cache::TtlCache<LenientHealthCheckRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientHealthCheckRuleStore for CachedLenientHealthCheckRuleStore {
    fn find(&self, code: &str) -> Option<LenientHealthCheckRule> {
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

    fn save(&self, item: &LenientHealthCheckRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientHealthCheckRule> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_health_check_rule_store(
    inner: Rc<dyn LenientHealthCheckRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientHealthCheckRuleStore> {
    Rc::new(CachedLenientHealthCheckRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientHealthCheckRule>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient health check rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientHealthCheckRuleQuery {
    pub unit: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientHealthCheckRuleQuery {
    pub fn matches(&self, item: &LenientHealthCheckRule) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientHealthCheckRule]) -> Vec<LenientHealthCheckRule> {
        let matched = collections::filter(items, |candidate: &LenientHealthCheckRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_health_check_rules(
    store: Rc<dyn LenientHealthCheckRuleStore>,
    query: &LenientHealthCheckRuleQuery,
) -> Vec<LenientHealthCheckRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientHealthCheckRuleStore> = new_memory_lenient_health_check_rule_store();
        let item = new_lenient_health_check_rule("code-819", "name-153");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
