use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps timeline requests by code.
pub trait TimelineRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TimelineRequest>;
    fn save(&self, item: &TimelineRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TimelineRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTimelineRequestStore {
    pub items: RefCell<BTreeMap<String, TimelineRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl TimelineRequestStore for MemoryTimelineRequestStore {
    fn find(&self, code: &str) -> Option<TimelineRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TimelineRequest) {
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

    fn list(&self) -> Vec<TimelineRequest> {
        let mut out: Vec<TimelineRequest> = Vec::new();
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

pub fn new_memory_timeline_request_store() -> Rc<dyn TimelineRequestStore> {
    Rc::new(MemoryTimelineRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTimelineRequestStore {
    pub inner: Rc<dyn TimelineRequestStore>,
    pub cache: RefCell<cache::TtlCache<TimelineRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TimelineRequestStore for CachedTimelineRequestStore {
    fn find(&self, code: &str) -> Option<TimelineRequest> {
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

    fn save(&self, item: &TimelineRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TimelineRequest> {
        self.inner.list()
    }
}

pub fn new_cached_timeline_request_store(
    inner: Rc<dyn TimelineRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TimelineRequestStore> {
    Rc::new(CachedTimelineRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TimelineRequest>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects timeline requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRequestQuery {
    pub source: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TimelineRequestQuery {
    pub fn matches(&self, item: &TimelineRequest) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TimelineRequest]) -> Vec<TimelineRequest> {
        let matched = collections::filter(items, |candidate: &TimelineRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_timeline_requests(
    store: Rc<dyn TimelineRequestStore>,
    query: &TimelineRequestQuery,
) -> Vec<TimelineRequest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TimelineRequestStore> = new_memory_timeline_request_store();
        let item = new_timeline_request("code-211", "title-527");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
