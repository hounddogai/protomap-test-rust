use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps identity batches by code.
pub trait IdentityBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<IdentityBatch>;
    fn save(&self, item: &IdentityBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<IdentityBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryIdentityBatchStore {
    pub items: RefCell<BTreeMap<String, IdentityBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl IdentityBatchStore for MemoryIdentityBatchStore {
    fn find(&self, code: &str) -> Option<IdentityBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &IdentityBatch) {
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

    fn list(&self) -> Vec<IdentityBatch> {
        let mut out: Vec<IdentityBatch> = Vec::new();
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

pub fn new_memory_identity_batch_store() -> Rc<dyn IdentityBatchStore> {
    Rc::new(MemoryIdentityBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedIdentityBatchStore {
    pub inner: Rc<dyn IdentityBatchStore>,
    pub cache: RefCell<cache::TtlCache<IdentityBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl IdentityBatchStore for CachedIdentityBatchStore {
    fn find(&self, code: &str) -> Option<IdentityBatch> {
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

    fn save(&self, item: &IdentityBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<IdentityBatch> {
        self.inner.list()
    }
}

pub fn new_cached_identity_batch_store(
    inner: Rc<dyn IdentityBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn IdentityBatchStore> {
    Rc::new(CachedIdentityBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<IdentityBatch>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects identity batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBatchQuery {
    pub label: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl IdentityBatchQuery {
    pub fn matches(&self, item: &IdentityBatch) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[IdentityBatch]) -> Vec<IdentityBatch> {
        let matched = collections::filter(items, |candidate: &IdentityBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_identity_batches(store: Rc<dyn IdentityBatchStore>, query: &IdentityBatchQuery) -> Vec<IdentityBatch> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn IdentityBatchStore> = new_memory_identity_batch_store();
        let item = new_identity_batch("code-628", "label-941");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
