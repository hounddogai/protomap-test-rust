use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy segment reviews by code.
pub trait LegacySegmentReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacySegmentReview>;
    fn save(&self, item: &LegacySegmentReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacySegmentReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacySegmentReviewStore {
    pub items: RefCell<BTreeMap<String, LegacySegmentReview>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacySegmentReviewStore for MemoryLegacySegmentReviewStore {
    fn find(&self, code: &str) -> Option<LegacySegmentReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacySegmentReview) {
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

    fn list(&self) -> Vec<LegacySegmentReview> {
        let mut out: Vec<LegacySegmentReview> = Vec::new();
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

pub fn new_memory_legacy_segment_review_store() -> Rc<dyn LegacySegmentReviewStore> {
    Rc::new(MemoryLegacySegmentReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacySegmentReviewStore {
    pub inner: Rc<dyn LegacySegmentReviewStore>,
    pub cache: RefCell<cache::TtlCache<LegacySegmentReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacySegmentReviewStore for CachedLegacySegmentReviewStore {
    fn find(&self, code: &str) -> Option<LegacySegmentReview> {
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

    fn save(&self, item: &LegacySegmentReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacySegmentReview> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_segment_review_store(
    inner: Rc<dyn LegacySegmentReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacySegmentReviewStore> {
    Rc::new(CachedLegacySegmentReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacySegmentReview>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy segment reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacySegmentReviewQuery {
    pub segment: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacySegmentReviewQuery {
    pub fn matches(&self, item: &LegacySegmentReview) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
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

    pub fn apply(&self, items: &[LegacySegmentReview]) -> Vec<LegacySegmentReview> {
        let matched = collections::filter(items, |candidate: &LegacySegmentReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_segment_reviews(
    store: Rc<dyn LegacySegmentReviewStore>,
    query: &LegacySegmentReviewQuery,
) -> Vec<LegacySegmentReview> {
    query.apply(&store.list())
}
