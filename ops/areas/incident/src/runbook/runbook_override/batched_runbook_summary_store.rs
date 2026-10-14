use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched runbook summaries by code.
pub trait BatchedRunbookSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedRunbookSummary>;
    fn save(&self, item: &BatchedRunbookSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedRunbookSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedRunbookSummaryStore {
    pub items: RefCell<BTreeMap<String, BatchedRunbookSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedRunbookSummaryStore for MemoryBatchedRunbookSummaryStore {
    fn find(&self, code: &str) -> Option<BatchedRunbookSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedRunbookSummary) {
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

    fn list(&self) -> Vec<BatchedRunbookSummary> {
        let mut out: Vec<BatchedRunbookSummary> = Vec::new();
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

pub fn new_memory_batched_runbook_summary_store() -> Rc<dyn BatchedRunbookSummaryStore> {
    Rc::new(MemoryBatchedRunbookSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedRunbookSummaryStore {
    pub inner: Rc<dyn BatchedRunbookSummaryStore>,
    pub cache: RefCell<cache::TtlCache<BatchedRunbookSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedRunbookSummaryStore for CachedBatchedRunbookSummaryStore {
    fn find(&self, code: &str) -> Option<BatchedRunbookSummary> {
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

    fn save(&self, item: &BatchedRunbookSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedRunbookSummary> {
        self.inner.list()
    }
}

pub fn new_cached_batched_runbook_summary_store(
    inner: Rc<dyn BatchedRunbookSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedRunbookSummaryStore> {
    Rc::new(CachedBatchedRunbookSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedRunbookSummary>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched runbook summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookSummaryQuery {
    pub name: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedRunbookSummaryQuery {
    pub fn matches(&self, item: &BatchedRunbookSummary) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedRunbookSummary]) -> Vec<BatchedRunbookSummary> {
        let matched = collections::filter(items, |candidate: &BatchedRunbookSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_runbook_summaries(
    store: Rc<dyn BatchedRunbookSummaryStore>,
    query: &BatchedRunbookSummaryQuery,
) -> Vec<BatchedRunbookSummary> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedRunbookSummaryStore> = new_memory_batched_runbook_summary_store();
        let item = new_batched_runbook_summary("code-723", "name-908");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
