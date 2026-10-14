use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched overage rules by code.
pub trait BatchedOverageRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedOverageRule>;
    fn save(&self, item: &BatchedOverageRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedOverageRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedOverageRuleStore {
    pub items: RefCell<BTreeMap<String, BatchedOverageRule>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedOverageRuleStore for MemoryBatchedOverageRuleStore {
    fn find(&self, code: &str) -> Option<BatchedOverageRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedOverageRule) {
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

    fn list(&self) -> Vec<BatchedOverageRule> {
        let mut out: Vec<BatchedOverageRule> = Vec::new();
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

pub fn new_memory_batched_overage_rule_store() -> Rc<dyn BatchedOverageRuleStore> {
    Rc::new(MemoryBatchedOverageRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedOverageRuleStore {
    pub inner: Rc<dyn BatchedOverageRuleStore>,
    pub cache: RefCell<cache::TtlCache<BatchedOverageRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedOverageRuleStore for CachedBatchedOverageRuleStore {
    fn find(&self, code: &str) -> Option<BatchedOverageRule> {
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

    fn save(&self, item: &BatchedOverageRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedOverageRule> {
        self.inner.list()
    }
}

pub fn new_cached_batched_overage_rule_store(
    inner: Rc<dyn BatchedOverageRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedOverageRuleStore> {
    Rc::new(CachedBatchedOverageRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedOverageRule>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched overage rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedOverageRuleQuery {
    pub name: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedOverageRuleQuery {
    pub fn matches(&self, item: &BatchedOverageRule) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedOverageRule]) -> Vec<BatchedOverageRule> {
        let matched = collections::filter(items, |candidate: &BatchedOverageRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_overage_rules(
    store: Rc<dyn BatchedOverageRuleStore>,
    query: &BatchedOverageRuleQuery,
) -> Vec<BatchedOverageRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedOverageRuleStore> = new_memory_batched_overage_rule_store();
        let item = new_batched_overage_rule("code-968", "name-956");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
