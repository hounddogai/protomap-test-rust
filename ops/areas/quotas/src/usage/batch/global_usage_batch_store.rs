use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global usage batches by code.
pub trait GlobalUsageBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalUsageBatch>;
    fn save(&self, item: &GlobalUsageBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalUsageBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalUsageBatchStore {
    pub items: RefCell<BTreeMap<String, GlobalUsageBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalUsageBatchStore for MemoryGlobalUsageBatchStore {
    fn find(&self, code: &str) -> Option<GlobalUsageBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalUsageBatch) {
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

    fn list(&self) -> Vec<GlobalUsageBatch> {
        let mut out: Vec<GlobalUsageBatch> = Vec::new();
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

pub fn new_memory_global_usage_batch_store() -> Rc<dyn GlobalUsageBatchStore> {
    Rc::new(MemoryGlobalUsageBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalUsageBatchStore {
    pub inner: Rc<dyn GlobalUsageBatchStore>,
    pub cache: RefCell<cache::TtlCache<GlobalUsageBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalUsageBatchStore for CachedGlobalUsageBatchStore {
    fn find(&self, code: &str) -> Option<GlobalUsageBatch> {
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

    fn save(&self, item: &GlobalUsageBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalUsageBatch> {
        self.inner.list()
    }
}

pub fn new_cached_global_usage_batch_store(
    inner: Rc<dyn GlobalUsageBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalUsageBatchStore> {
    Rc::new(CachedGlobalUsageBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalUsageBatch>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global usage batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsageBatchQuery {
    pub label: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalUsageBatchQuery {
    pub fn matches(&self, item: &GlobalUsageBatch) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.cost < self.min_cost {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalUsageBatch]) -> Vec<GlobalUsageBatch> {
        let matched = collections::filter(items, |candidate: &GlobalUsageBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_usage_batches(
    store: Rc<dyn GlobalUsageBatchStore>,
    query: &GlobalUsageBatchQuery,
) -> Vec<GlobalUsageBatch> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalUsageBatchStore> = new_memory_global_usage_batch_store();
        let item = new_global_usage_batch("code-145", "label-93");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
