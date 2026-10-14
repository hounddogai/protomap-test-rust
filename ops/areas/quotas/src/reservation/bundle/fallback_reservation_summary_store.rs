use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback reservation summaries by code.
pub trait FallbackReservationSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackReservationSummary>;
    fn save(&self, item: &FallbackReservationSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackReservationSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackReservationSummaryStore {
    pub items: RefCell<BTreeMap<String, FallbackReservationSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackReservationSummaryStore for MemoryFallbackReservationSummaryStore {
    fn find(&self, code: &str) -> Option<FallbackReservationSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackReservationSummary) {
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

    fn list(&self) -> Vec<FallbackReservationSummary> {
        let mut out: Vec<FallbackReservationSummary> = Vec::new();
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

pub fn new_memory_fallback_reservation_summary_store() -> Rc<dyn FallbackReservationSummaryStore> {
    Rc::new(MemoryFallbackReservationSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackReservationSummaryStore {
    pub inner: Rc<dyn FallbackReservationSummaryStore>,
    pub cache: RefCell<cache::TtlCache<FallbackReservationSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackReservationSummaryStore for CachedFallbackReservationSummaryStore {
    fn find(&self, code: &str) -> Option<FallbackReservationSummary> {
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

    fn save(&self, item: &FallbackReservationSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackReservationSummary> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_reservation_summary_store(
    inner: Rc<dyn FallbackReservationSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackReservationSummaryStore> {
    Rc::new(CachedFallbackReservationSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackReservationSummary>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback reservation summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationSummaryQuery {
    pub locale: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackReservationSummaryQuery {
    pub fn matches(&self, item: &FallbackReservationSummary) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackReservationSummary]) -> Vec<FallbackReservationSummary> {
        let matched = collections::filter(items, |candidate: &FallbackReservationSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_reservation_summaries(
    store: Rc<dyn FallbackReservationSummaryStore>,
    query: &FallbackReservationSummaryQuery,
) -> Vec<FallbackReservationSummary> {
    query.apply(&store.list())
}
