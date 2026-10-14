use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global rollout rules by code.
pub trait GlobalRolloutRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalRolloutRule>;
    fn save(&self, item: &GlobalRolloutRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalRolloutRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalRolloutRuleStore {
    pub items: RefCell<BTreeMap<String, GlobalRolloutRule>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalRolloutRuleStore for MemoryGlobalRolloutRuleStore {
    fn find(&self, code: &str) -> Option<GlobalRolloutRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalRolloutRule) {
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

    fn list(&self) -> Vec<GlobalRolloutRule> {
        let mut out: Vec<GlobalRolloutRule> = Vec::new();
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

pub fn new_memory_global_rollout_rule_store() -> Rc<dyn GlobalRolloutRuleStore> {
    Rc::new(MemoryGlobalRolloutRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalRolloutRuleStore {
    pub inner: Rc<dyn GlobalRolloutRuleStore>,
    pub cache: RefCell<cache::TtlCache<GlobalRolloutRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalRolloutRuleStore for CachedGlobalRolloutRuleStore {
    fn find(&self, code: &str) -> Option<GlobalRolloutRule> {
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

    fn save(&self, item: &GlobalRolloutRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalRolloutRule> {
        self.inner.list()
    }
}

pub fn new_cached_global_rollout_rule_store(
    inner: Rc<dyn GlobalRolloutRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalRolloutRuleStore> {
    Rc::new(CachedGlobalRolloutRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalRolloutRule>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global rollout rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutRuleQuery {
    pub locale: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalRolloutRuleQuery {
    pub fn matches(&self, item: &GlobalRolloutRule) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalRolloutRule]) -> Vec<GlobalRolloutRule> {
        let matched = collections::filter(items, |candidate: &GlobalRolloutRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_rollout_rules(
    store: Rc<dyn GlobalRolloutRuleStore>,
    query: &GlobalRolloutRuleQuery,
) -> Vec<GlobalRolloutRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalRolloutRuleStore> = new_memory_global_rollout_rule_store();
        let item = new_global_rollout_rule("code-312", "label-270");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
