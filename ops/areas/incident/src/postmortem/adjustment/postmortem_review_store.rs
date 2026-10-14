use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps postmortem reviews by code.
pub trait PostmortemReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PostmortemReview>;
    fn save(&self, item: &PostmortemReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PostmortemReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPostmortemReviewStore {
    pub items: RefCell<BTreeMap<String, PostmortemReview>>,
    pub order: RefCell<Vec<String>>,
}

impl PostmortemReviewStore for MemoryPostmortemReviewStore {
    fn find(&self, code: &str) -> Option<PostmortemReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PostmortemReview) {
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

    fn list(&self) -> Vec<PostmortemReview> {
        let mut out: Vec<PostmortemReview> = Vec::new();
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

pub fn new_memory_postmortem_review_store() -> Rc<dyn PostmortemReviewStore> {
    Rc::new(MemoryPostmortemReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPostmortemReviewStore {
    pub inner: Rc<dyn PostmortemReviewStore>,
    pub cache: RefCell<cache::TtlCache<PostmortemReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PostmortemReviewStore for CachedPostmortemReviewStore {
    fn find(&self, code: &str) -> Option<PostmortemReview> {
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

    fn save(&self, item: &PostmortemReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PostmortemReview> {
        self.inner.list()
    }
}

pub fn new_cached_postmortem_review_store(
    inner: Rc<dyn PostmortemReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PostmortemReviewStore> {
    Rc::new(CachedPostmortemReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PostmortemReview>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects postmortem reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemReviewQuery {
    pub title: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PostmortemReviewQuery {
    pub fn matches(&self, item: &PostmortemReview) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PostmortemReview]) -> Vec<PostmortemReview> {
        let matched = collections::filter(items, |candidate: &PostmortemReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_postmortem_reviews(
    store: Rc<dyn PostmortemReviewStore>,
    query: &PostmortemReviewQuery,
) -> Vec<PostmortemReview> {
    query.apply(&store.list())
}
