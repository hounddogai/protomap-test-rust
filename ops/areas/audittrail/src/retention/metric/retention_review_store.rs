use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps retention reviews by code.
pub trait RetentionReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RetentionReview>;
    fn save(&self, item: &RetentionReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RetentionReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRetentionReviewStore {
    pub items: RefCell<BTreeMap<String, RetentionReview>>,
    pub order: RefCell<Vec<String>>,
}

impl RetentionReviewStore for MemoryRetentionReviewStore {
    fn find(&self, code: &str) -> Option<RetentionReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RetentionReview) {
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

    fn list(&self) -> Vec<RetentionReview> {
        let mut out: Vec<RetentionReview> = Vec::new();
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

pub fn new_memory_retention_review_store() -> Rc<dyn RetentionReviewStore> {
    Rc::new(MemoryRetentionReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRetentionReviewStore {
    pub inner: Rc<dyn RetentionReviewStore>,
    pub cache: RefCell<cache::TtlCache<RetentionReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RetentionReviewStore for CachedRetentionReviewStore {
    fn find(&self, code: &str) -> Option<RetentionReview> {
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

    fn save(&self, item: &RetentionReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RetentionReview> {
        self.inner.list()
    }
}

pub fn new_cached_retention_review_store(
    inner: Rc<dyn RetentionReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RetentionReviewStore> {
    Rc::new(CachedRetentionReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RetentionReview>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects retention reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionReviewQuery {
    pub label: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RetentionReviewQuery {
    pub fn matches(&self, item: &RetentionReview) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[RetentionReview]) -> Vec<RetentionReview> {
        let matched = collections::filter(items, |candidate: &RetentionReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_retention_reviews(
    store: Rc<dyn RetentionReviewStore>,
    query: &RetentionReviewQuery,
) -> Vec<RetentionReview> {
    query.apply(&store.list())
}
