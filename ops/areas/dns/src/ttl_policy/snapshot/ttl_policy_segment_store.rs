use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps ttl policy segments by code.
pub trait TtlPolicySegmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TtlPolicySegment>;
    fn save(&self, item: &TtlPolicySegment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TtlPolicySegment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTtlPolicySegmentStore {
    pub items: RefCell<BTreeMap<String, TtlPolicySegment>>,
    pub order: RefCell<Vec<String>>,
}

impl TtlPolicySegmentStore for MemoryTtlPolicySegmentStore {
    fn find(&self, code: &str) -> Option<TtlPolicySegment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TtlPolicySegment) {
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

    fn list(&self) -> Vec<TtlPolicySegment> {
        let mut out: Vec<TtlPolicySegment> = Vec::new();
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

pub fn new_memory_ttl_policy_segment_store() -> Rc<dyn TtlPolicySegmentStore> {
    Rc::new(MemoryTtlPolicySegmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTtlPolicySegmentStore {
    pub inner: Rc<dyn TtlPolicySegmentStore>,
    pub cache: RefCell<cache::TtlCache<TtlPolicySegment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TtlPolicySegmentStore for CachedTtlPolicySegmentStore {
    fn find(&self, code: &str) -> Option<TtlPolicySegment> {
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

    fn save(&self, item: &TtlPolicySegment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TtlPolicySegment> {
        self.inner.list()
    }
}

pub fn new_cached_ttl_policy_segment_store(
    inner: Rc<dyn TtlPolicySegmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TtlPolicySegmentStore> {
    Rc::new(CachedTtlPolicySegmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TtlPolicySegment>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects ttl policy segments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicySegmentQuery {
    pub name: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TtlPolicySegmentQuery {
    pub fn matches(&self, item: &TtlPolicySegment) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TtlPolicySegment]) -> Vec<TtlPolicySegment> {
        let matched = collections::filter(items, |candidate: &TtlPolicySegment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_ttl_policy_segments(
    store: Rc<dyn TtlPolicySegmentStore>,
    query: &TtlPolicySegmentQuery,
) -> Vec<TtlPolicySegment> {
    query.apply(&store.list())
}
