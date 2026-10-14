use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps measurement reviews by code.
pub trait MeasurementReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MeasurementReview>;
    fn save(&self, item: &MeasurementReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MeasurementReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMeasurementReviewStore {
    pub items: RefCell<BTreeMap<String, MeasurementReview>>,
    pub order: RefCell<Vec<String>>,
}

impl MeasurementReviewStore for MemoryMeasurementReviewStore {
    fn find(&self, code: &str) -> Option<MeasurementReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MeasurementReview) {
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

    fn list(&self) -> Vec<MeasurementReview> {
        let mut out: Vec<MeasurementReview> = Vec::new();
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

pub fn new_memory_measurement_review_store() -> Rc<dyn MeasurementReviewStore> {
    Rc::new(MemoryMeasurementReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMeasurementReviewStore {
    pub inner: Rc<dyn MeasurementReviewStore>,
    pub cache: RefCell<cache::TtlCache<MeasurementReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MeasurementReviewStore for CachedMeasurementReviewStore {
    fn find(&self, code: &str) -> Option<MeasurementReview> {
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

    fn save(&self, item: &MeasurementReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MeasurementReview> {
        self.inner.list()
    }
}

pub fn new_cached_measurement_review_store(
    inner: Rc<dyn MeasurementReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MeasurementReviewStore> {
    Rc::new(CachedMeasurementReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MeasurementReview>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects measurement reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementReviewQuery {
    pub category: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MeasurementReviewQuery {
    pub fn matches(&self, item: &MeasurementReview) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MeasurementReview]) -> Vec<MeasurementReview> {
        let matched = collections::filter(items, |candidate: &MeasurementReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_measurement_reviews(
    store: Rc<dyn MeasurementReviewStore>,
    query: &MeasurementReviewQuery,
) -> Vec<MeasurementReview> {
    query.apply(&store.list())
}
