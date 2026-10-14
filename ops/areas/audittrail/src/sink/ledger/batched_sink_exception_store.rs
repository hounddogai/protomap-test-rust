use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched sink exceptions by code.
pub trait BatchedSinkExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedSinkException>;
    fn save(&self, item: &BatchedSinkException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedSinkException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedSinkExceptionStore {
    pub items: RefCell<BTreeMap<String, BatchedSinkException>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedSinkExceptionStore for MemoryBatchedSinkExceptionStore {
    fn find(&self, code: &str) -> Option<BatchedSinkException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedSinkException) {
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

    fn list(&self) -> Vec<BatchedSinkException> {
        let mut out: Vec<BatchedSinkException> = Vec::new();
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

pub fn new_memory_batched_sink_exception_store() -> Rc<dyn BatchedSinkExceptionStore> {
    Rc::new(MemoryBatchedSinkExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedSinkExceptionStore {
    pub inner: Rc<dyn BatchedSinkExceptionStore>,
    pub cache: RefCell<cache::TtlCache<BatchedSinkException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedSinkExceptionStore for CachedBatchedSinkExceptionStore {
    fn find(&self, code: &str) -> Option<BatchedSinkException> {
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

    fn save(&self, item: &BatchedSinkException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedSinkException> {
        self.inner.list()
    }
}

pub fn new_cached_batched_sink_exception_store(
    inner: Rc<dyn BatchedSinkExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedSinkExceptionStore> {
    Rc::new(CachedBatchedSinkExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedSinkException>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched sink exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkExceptionQuery {
    pub region: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedSinkExceptionQuery {
    pub fn matches(&self, item: &BatchedSinkException) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[BatchedSinkException]) -> Vec<BatchedSinkException> {
        let matched = collections::filter(items, |candidate: &BatchedSinkException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_sink_exceptions(
    store: Rc<dyn BatchedSinkExceptionStore>,
    query: &BatchedSinkExceptionQuery,
) -> Vec<BatchedSinkException> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedSinkExceptionStore> = new_memory_batched_sink_exception_store();
        let item = new_batched_sink_exception("code-697", "title-169");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
