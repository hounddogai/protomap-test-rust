use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive rule batches by code.
pub trait AdaptiveRuleBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveRuleBatch>;
    fn save(&self, item: &AdaptiveRuleBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveRuleBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveRuleBatchStore {
    pub items: RefCell<BTreeMap<String, AdaptiveRuleBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveRuleBatchStore for MemoryAdaptiveRuleBatchStore {
    fn find(&self, code: &str) -> Option<AdaptiveRuleBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveRuleBatch) {
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

    fn list(&self) -> Vec<AdaptiveRuleBatch> {
        let mut out: Vec<AdaptiveRuleBatch> = Vec::new();
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

pub fn new_memory_adaptive_rule_batch_store() -> Rc<dyn AdaptiveRuleBatchStore> {
    Rc::new(MemoryAdaptiveRuleBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveRuleBatchStore {
    pub inner: Rc<dyn AdaptiveRuleBatchStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveRuleBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveRuleBatchStore for CachedAdaptiveRuleBatchStore {
    fn find(&self, code: &str) -> Option<AdaptiveRuleBatch> {
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

    fn save(&self, item: &AdaptiveRuleBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveRuleBatch> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_rule_batch_store(
    inner: Rc<dyn AdaptiveRuleBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveRuleBatchStore> {
    Rc::new(CachedAdaptiveRuleBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveRuleBatch>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive rule batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleBatchQuery {
    pub channel: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveRuleBatchQuery {
    pub fn matches(&self, item: &AdaptiveRuleBatch) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveRuleBatch]) -> Vec<AdaptiveRuleBatch> {
        let matched = collections::filter(items, |candidate: &AdaptiveRuleBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_rule_batches(
    store: Rc<dyn AdaptiveRuleBatchStore>,
    query: &AdaptiveRuleBatchQuery,
) -> Vec<AdaptiveRuleBatch> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AdaptiveRuleBatchStore> = new_memory_adaptive_rule_batch_store();
        let item = new_adaptive_rule_batch("code-799", "label-215");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
