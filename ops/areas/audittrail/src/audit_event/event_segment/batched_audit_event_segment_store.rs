use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched audit event segments by code.
pub trait BatchedAuditEventSegmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedAuditEventSegment>;
    fn save(&self, item: &BatchedAuditEventSegment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedAuditEventSegment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedAuditEventSegmentStore {
    pub items: RefCell<BTreeMap<String, BatchedAuditEventSegment>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedAuditEventSegmentStore for MemoryBatchedAuditEventSegmentStore {
    fn find(&self, code: &str) -> Option<BatchedAuditEventSegment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedAuditEventSegment) {
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

    fn list(&self) -> Vec<BatchedAuditEventSegment> {
        let mut out: Vec<BatchedAuditEventSegment> = Vec::new();
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

pub fn new_memory_batched_audit_event_segment_store() -> Rc<dyn BatchedAuditEventSegmentStore> {
    Rc::new(MemoryBatchedAuditEventSegmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedAuditEventSegmentStore {
    pub inner: Rc<dyn BatchedAuditEventSegmentStore>,
    pub cache: RefCell<cache::TtlCache<BatchedAuditEventSegment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedAuditEventSegmentStore for CachedBatchedAuditEventSegmentStore {
    fn find(&self, code: &str) -> Option<BatchedAuditEventSegment> {
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

    fn save(&self, item: &BatchedAuditEventSegment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedAuditEventSegment> {
        self.inner.list()
    }
}

pub fn new_cached_batched_audit_event_segment_store(
    inner: Rc<dyn BatchedAuditEventSegmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedAuditEventSegmentStore> {
    Rc::new(CachedBatchedAuditEventSegmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedAuditEventSegment>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched audit event segments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventSegmentQuery {
    pub title: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedAuditEventSegmentQuery {
    pub fn matches(&self, item: &BatchedAuditEventSegment) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[BatchedAuditEventSegment]) -> Vec<BatchedAuditEventSegment> {
        let matched = collections::filter(items, |candidate: &BatchedAuditEventSegment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_audit_event_segments(
    store: Rc<dyn BatchedAuditEventSegmentStore>,
    query: &BatchedAuditEventSegmentQuery,
) -> Vec<BatchedAuditEventSegment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedAuditEventSegmentStore> = new_memory_batched_audit_event_segment_store();
        let item = new_batched_audit_event_segment("code-925", "title-973");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
