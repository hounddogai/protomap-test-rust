use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps timeline drafts by code.
pub trait TimelineDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TimelineDraft>;
    fn save(&self, item: &TimelineDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TimelineDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTimelineDraftStore {
    pub items: RefCell<BTreeMap<String, TimelineDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl TimelineDraftStore for MemoryTimelineDraftStore {
    fn find(&self, code: &str) -> Option<TimelineDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TimelineDraft) {
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

    fn list(&self) -> Vec<TimelineDraft> {
        let mut out: Vec<TimelineDraft> = Vec::new();
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

pub fn new_memory_timeline_draft_store() -> Rc<dyn TimelineDraftStore> {
    Rc::new(MemoryTimelineDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTimelineDraftStore {
    pub inner: Rc<dyn TimelineDraftStore>,
    pub cache: RefCell<cache::TtlCache<TimelineDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TimelineDraftStore for CachedTimelineDraftStore {
    fn find(&self, code: &str) -> Option<TimelineDraft> {
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

    fn save(&self, item: &TimelineDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TimelineDraft> {
        self.inner.list()
    }
}

pub fn new_cached_timeline_draft_store(
    inner: Rc<dyn TimelineDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TimelineDraftStore> {
    Rc::new(CachedTimelineDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TimelineDraft>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects timeline drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineDraftQuery {
    pub name: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TimelineDraftQuery {
    pub fn matches(&self, item: &TimelineDraft) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TimelineDraft]) -> Vec<TimelineDraft> {
        let matched = collections::filter(items, |candidate: &TimelineDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_timeline_drafts(store: Rc<dyn TimelineDraftStore>, query: &TimelineDraftQuery) -> Vec<TimelineDraft> {
    query.apply(&store.list())
}
