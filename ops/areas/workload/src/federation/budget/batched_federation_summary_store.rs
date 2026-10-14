use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched federation summaries by code.
pub trait BatchedFederationSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedFederationSummary>;
    fn save(&self, item: &BatchedFederationSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedFederationSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedFederationSummaryStore {
    pub items: RefCell<BTreeMap<String, BatchedFederationSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedFederationSummaryStore for MemoryBatchedFederationSummaryStore {
    fn find(&self, code: &str) -> Option<BatchedFederationSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedFederationSummary) {
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

    fn list(&self) -> Vec<BatchedFederationSummary> {
        let mut out: Vec<BatchedFederationSummary> = Vec::new();
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

pub fn new_memory_batched_federation_summary_store() -> Rc<dyn BatchedFederationSummaryStore> {
    Rc::new(MemoryBatchedFederationSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedFederationSummaryStore {
    pub inner: Rc<dyn BatchedFederationSummaryStore>,
    pub cache: RefCell<cache::TtlCache<BatchedFederationSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedFederationSummaryStore for CachedBatchedFederationSummaryStore {
    fn find(&self, code: &str) -> Option<BatchedFederationSummary> {
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

    fn save(&self, item: &BatchedFederationSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedFederationSummary> {
        self.inner.list()
    }
}

pub fn new_cached_batched_federation_summary_store(
    inner: Rc<dyn BatchedFederationSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedFederationSummaryStore> {
    Rc::new(CachedBatchedFederationSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedFederationSummary>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched federation summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationSummaryQuery {
    pub region: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedFederationSummaryQuery {
    pub fn matches(&self, item: &BatchedFederationSummary) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedFederationSummary]) -> Vec<BatchedFederationSummary> {
        let matched = collections::filter(items, |candidate: &BatchedFederationSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_federation_summaries(
    store: Rc<dyn BatchedFederationSummaryStore>,
    query: &BatchedFederationSummaryQuery,
) -> Vec<BatchedFederationSummary> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedFederationSummaryStore> = new_memory_batched_federation_summary_store();
        let item = new_batched_federation_summary("code-345", "name-769");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
