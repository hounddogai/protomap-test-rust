use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched redaction summaries by code.
pub trait BatchedRedactionSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedRedactionSummary>;
    fn save(&self, item: &BatchedRedactionSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedRedactionSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedRedactionSummaryStore {
    pub items: RefCell<BTreeMap<String, BatchedRedactionSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedRedactionSummaryStore for MemoryBatchedRedactionSummaryStore {
    fn find(&self, code: &str) -> Option<BatchedRedactionSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedRedactionSummary) {
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

    fn list(&self) -> Vec<BatchedRedactionSummary> {
        let mut out: Vec<BatchedRedactionSummary> = Vec::new();
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

pub fn new_memory_batched_redaction_summary_store() -> Rc<dyn BatchedRedactionSummaryStore> {
    Rc::new(MemoryBatchedRedactionSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedRedactionSummaryStore {
    pub inner: Rc<dyn BatchedRedactionSummaryStore>,
    pub cache: RefCell<cache::TtlCache<BatchedRedactionSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedRedactionSummaryStore for CachedBatchedRedactionSummaryStore {
    fn find(&self, code: &str) -> Option<BatchedRedactionSummary> {
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

    fn save(&self, item: &BatchedRedactionSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedRedactionSummary> {
        self.inner.list()
    }
}

pub fn new_cached_batched_redaction_summary_store(
    inner: Rc<dyn BatchedRedactionSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedRedactionSummaryStore> {
    Rc::new(CachedBatchedRedactionSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedRedactionSummary>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched redaction summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionSummaryQuery {
    pub category: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedRedactionSummaryQuery {
    pub fn matches(&self, item: &BatchedRedactionSummary) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedRedactionSummary]) -> Vec<BatchedRedactionSummary> {
        let matched = collections::filter(items, |candidate: &BatchedRedactionSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_redaction_summaries(
    store: Rc<dyn BatchedRedactionSummaryStore>,
    query: &BatchedRedactionSummaryQuery,
) -> Vec<BatchedRedactionSummary> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedRedactionSummaryStore> = new_memory_batched_redaction_summary_store();
        let item = new_batched_redaction_summary("code-58", "label-111");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
