use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled failover segments by code.
pub trait ScheduledFailoverSegmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledFailoverSegment>;
    fn save(&self, item: &ScheduledFailoverSegment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledFailoverSegment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledFailoverSegmentStore {
    pub items: RefCell<BTreeMap<String, ScheduledFailoverSegment>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledFailoverSegmentStore for MemoryScheduledFailoverSegmentStore {
    fn find(&self, code: &str) -> Option<ScheduledFailoverSegment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledFailoverSegment) {
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

    fn list(&self) -> Vec<ScheduledFailoverSegment> {
        let mut out: Vec<ScheduledFailoverSegment> = Vec::new();
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

pub fn new_memory_scheduled_failover_segment_store() -> Rc<dyn ScheduledFailoverSegmentStore> {
    Rc::new(MemoryScheduledFailoverSegmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledFailoverSegmentStore {
    pub inner: Rc<dyn ScheduledFailoverSegmentStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledFailoverSegment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledFailoverSegmentStore for CachedScheduledFailoverSegmentStore {
    fn find(&self, code: &str) -> Option<ScheduledFailoverSegment> {
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

    fn save(&self, item: &ScheduledFailoverSegment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledFailoverSegment> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_failover_segment_store(
    inner: Rc<dyn ScheduledFailoverSegmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledFailoverSegmentStore> {
    Rc::new(CachedScheduledFailoverSegmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledFailoverSegment>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled failover segments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverSegmentQuery {
    pub segment: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledFailoverSegmentQuery {
    pub fn matches(&self, item: &ScheduledFailoverSegment) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledFailoverSegment]) -> Vec<ScheduledFailoverSegment> {
        let matched = collections::filter(items, |candidate: &ScheduledFailoverSegment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_failover_segments(
    store: Rc<dyn ScheduledFailoverSegmentStore>,
    query: &ScheduledFailoverSegmentQuery,
) -> Vec<ScheduledFailoverSegment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ScheduledFailoverSegmentStore> = new_memory_scheduled_failover_segment_store();
        let item = new_scheduled_failover_segment("code-414", "title-62");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
