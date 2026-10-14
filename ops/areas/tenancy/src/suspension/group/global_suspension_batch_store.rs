use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global suspension batches by code.
pub trait GlobalSuspensionBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalSuspensionBatch>;
    fn save(&self, item: &GlobalSuspensionBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalSuspensionBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalSuspensionBatchStore {
    pub items: RefCell<BTreeMap<String, GlobalSuspensionBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalSuspensionBatchStore for MemoryGlobalSuspensionBatchStore {
    fn find(&self, code: &str) -> Option<GlobalSuspensionBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalSuspensionBatch) {
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

    fn list(&self) -> Vec<GlobalSuspensionBatch> {
        let mut out: Vec<GlobalSuspensionBatch> = Vec::new();
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

pub fn new_memory_global_suspension_batch_store() -> Rc<dyn GlobalSuspensionBatchStore> {
    Rc::new(MemoryGlobalSuspensionBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalSuspensionBatchStore {
    pub inner: Rc<dyn GlobalSuspensionBatchStore>,
    pub cache: RefCell<cache::TtlCache<GlobalSuspensionBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalSuspensionBatchStore for CachedGlobalSuspensionBatchStore {
    fn find(&self, code: &str) -> Option<GlobalSuspensionBatch> {
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

    fn save(&self, item: &GlobalSuspensionBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalSuspensionBatch> {
        self.inner.list()
    }
}

pub fn new_cached_global_suspension_batch_store(
    inner: Rc<dyn GlobalSuspensionBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalSuspensionBatchStore> {
    Rc::new(CachedGlobalSuspensionBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalSuspensionBatch>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global suspension batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSuspensionBatchQuery {
    pub label: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalSuspensionBatchQuery {
    pub fn matches(&self, item: &GlobalSuspensionBatch) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalSuspensionBatch]) -> Vec<GlobalSuspensionBatch> {
        let matched = collections::filter(items, |candidate: &GlobalSuspensionBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_suspension_batches(
    store: Rc<dyn GlobalSuspensionBatchStore>,
    query: &GlobalSuspensionBatchQuery,
) -> Vec<GlobalSuspensionBatch> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalSuspensionBatchStore> = new_memory_global_suspension_batch_store();
        let item = new_global_suspension_batch("code-633", "label-28");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
