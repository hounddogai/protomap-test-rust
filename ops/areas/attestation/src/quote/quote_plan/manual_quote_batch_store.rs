use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual quote batches by code.
pub trait ManualQuoteBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualQuoteBatch>;
    fn save(&self, item: &ManualQuoteBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualQuoteBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualQuoteBatchStore {
    pub items: RefCell<BTreeMap<String, ManualQuoteBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualQuoteBatchStore for MemoryManualQuoteBatchStore {
    fn find(&self, code: &str) -> Option<ManualQuoteBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualQuoteBatch) {
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

    fn list(&self) -> Vec<ManualQuoteBatch> {
        let mut out: Vec<ManualQuoteBatch> = Vec::new();
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

pub fn new_memory_manual_quote_batch_store() -> Rc<dyn ManualQuoteBatchStore> {
    Rc::new(MemoryManualQuoteBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualQuoteBatchStore {
    pub inner: Rc<dyn ManualQuoteBatchStore>,
    pub cache: RefCell<cache::TtlCache<ManualQuoteBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualQuoteBatchStore for CachedManualQuoteBatchStore {
    fn find(&self, code: &str) -> Option<ManualQuoteBatch> {
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

    fn save(&self, item: &ManualQuoteBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualQuoteBatch> {
        self.inner.list()
    }
}

pub fn new_cached_manual_quote_batch_store(
    inner: Rc<dyn ManualQuoteBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualQuoteBatchStore> {
    Rc::new(CachedManualQuoteBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualQuoteBatch>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual quote batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuoteBatchQuery {
    pub namespace: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualQuoteBatchQuery {
    pub fn matches(&self, item: &ManualQuoteBatch) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualQuoteBatch]) -> Vec<ManualQuoteBatch> {
        let matched = collections::filter(items, |candidate: &ManualQuoteBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_quote_batches(
    store: Rc<dyn ManualQuoteBatchStore>,
    query: &ManualQuoteBatchQuery,
) -> Vec<ManualQuoteBatch> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualQuoteBatchStore> = new_memory_manual_quote_batch_store();
        let item = new_manual_quote_batch("code-798", "title-379");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
