use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted freeze reviews by code.
pub trait WeightedFreezeReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedFreezeReview>;
    fn save(&self, item: &WeightedFreezeReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedFreezeReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedFreezeReviewStore {
    pub items: RefCell<BTreeMap<String, WeightedFreezeReview>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedFreezeReviewStore for MemoryWeightedFreezeReviewStore {
    fn find(&self, code: &str) -> Option<WeightedFreezeReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedFreezeReview) {
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

    fn list(&self) -> Vec<WeightedFreezeReview> {
        let mut out: Vec<WeightedFreezeReview> = Vec::new();
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

pub fn new_memory_weighted_freeze_review_store() -> Rc<dyn WeightedFreezeReviewStore> {
    Rc::new(MemoryWeightedFreezeReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedFreezeReviewStore {
    pub inner: Rc<dyn WeightedFreezeReviewStore>,
    pub cache: RefCell<cache::TtlCache<WeightedFreezeReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedFreezeReviewStore for CachedWeightedFreezeReviewStore {
    fn find(&self, code: &str) -> Option<WeightedFreezeReview> {
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

    fn save(&self, item: &WeightedFreezeReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedFreezeReview> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_freeze_review_store(
    inner: Rc<dyn WeightedFreezeReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedFreezeReviewStore> {
    Rc::new(CachedWeightedFreezeReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedFreezeReview>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted freeze reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFreezeReviewQuery {
    pub label: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedFreezeReviewQuery {
    pub fn matches(&self, item: &WeightedFreezeReview) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedFreezeReview]) -> Vec<WeightedFreezeReview> {
        let matched = collections::filter(items, |candidate: &WeightedFreezeReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_freeze_reviews(
    store: Rc<dyn WeightedFreezeReviewStore>,
    query: &WeightedFreezeReviewQuery,
) -> Vec<WeightedFreezeReview> {
    query.apply(&store.list())
}
