use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps action item reviews by code.
pub trait ActionItemReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ActionItemReview>;
    fn save(&self, item: &ActionItemReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ActionItemReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryActionItemReviewStore {
    pub items: RefCell<BTreeMap<String, ActionItemReview>>,
    pub order: RefCell<Vec<String>>,
}

impl ActionItemReviewStore for MemoryActionItemReviewStore {
    fn find(&self, code: &str) -> Option<ActionItemReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ActionItemReview) {
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

    fn list(&self) -> Vec<ActionItemReview> {
        let mut out: Vec<ActionItemReview> = Vec::new();
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

pub fn new_memory_action_item_review_store() -> Rc<dyn ActionItemReviewStore> {
    Rc::new(MemoryActionItemReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedActionItemReviewStore {
    pub inner: Rc<dyn ActionItemReviewStore>,
    pub cache: RefCell<cache::TtlCache<ActionItemReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ActionItemReviewStore for CachedActionItemReviewStore {
    fn find(&self, code: &str) -> Option<ActionItemReview> {
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

    fn save(&self, item: &ActionItemReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ActionItemReview> {
        self.inner.list()
    }
}

pub fn new_cached_action_item_review_store(
    inner: Rc<dyn ActionItemReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ActionItemReviewStore> {
    Rc::new(CachedActionItemReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ActionItemReview>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects action item reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemReviewQuery {
    pub label: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ActionItemReviewQuery {
    pub fn matches(&self, item: &ActionItemReview) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ActionItemReview]) -> Vec<ActionItemReview> {
        let matched = collections::filter(items, |candidate: &ActionItemReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_action_item_reviews(
    store: Rc<dyn ActionItemReviewStore>,
    query: &ActionItemReviewQuery,
) -> Vec<ActionItemReview> {
    query.apply(&store.list())
}
