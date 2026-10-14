use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps timeline claims by code.
pub trait TimelineClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TimelineClaim>;
    fn save(&self, item: &TimelineClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TimelineClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTimelineClaimStore {
    pub items: RefCell<BTreeMap<String, TimelineClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl TimelineClaimStore for MemoryTimelineClaimStore {
    fn find(&self, code: &str) -> Option<TimelineClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TimelineClaim) {
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

    fn list(&self) -> Vec<TimelineClaim> {
        let mut out: Vec<TimelineClaim> = Vec::new();
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

pub fn new_memory_timeline_claim_store() -> Rc<dyn TimelineClaimStore> {
    Rc::new(MemoryTimelineClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTimelineClaimStore {
    pub inner: Rc<dyn TimelineClaimStore>,
    pub cache: RefCell<cache::TtlCache<TimelineClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TimelineClaimStore for CachedTimelineClaimStore {
    fn find(&self, code: &str) -> Option<TimelineClaim> {
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

    fn save(&self, item: &TimelineClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TimelineClaim> {
        self.inner.list()
    }
}

pub fn new_cached_timeline_claim_store(
    inner: Rc<dyn TimelineClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TimelineClaimStore> {
    Rc::new(CachedTimelineClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TimelineClaim>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects timeline claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineClaimQuery {
    pub source: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TimelineClaimQuery {
    pub fn matches(&self, item: &TimelineClaim) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TimelineClaim]) -> Vec<TimelineClaim> {
        let matched = collections::filter(items, |candidate: &TimelineClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_timeline_claims(store: Rc<dyn TimelineClaimStore>, query: &TimelineClaimQuery) -> Vec<TimelineClaim> {
    query.apply(&store.list())
}
