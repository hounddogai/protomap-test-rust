use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps nonce reviews by code.
pub trait NonceReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<NonceReview>;
    fn save(&self, item: &NonceReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<NonceReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryNonceReviewStore {
    pub items: RefCell<BTreeMap<String, NonceReview>>,
    pub order: RefCell<Vec<String>>,
}

impl NonceReviewStore for MemoryNonceReviewStore {
    fn find(&self, code: &str) -> Option<NonceReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &NonceReview) {
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

    fn list(&self) -> Vec<NonceReview> {
        let mut out: Vec<NonceReview> = Vec::new();
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

pub fn new_memory_nonce_review_store() -> Rc<dyn NonceReviewStore> {
    Rc::new(MemoryNonceReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedNonceReviewStore {
    pub inner: Rc<dyn NonceReviewStore>,
    pub cache: RefCell<cache::TtlCache<NonceReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl NonceReviewStore for CachedNonceReviewStore {
    fn find(&self, code: &str) -> Option<NonceReview> {
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

    fn save(&self, item: &NonceReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<NonceReview> {
        self.inner.list()
    }
}

pub fn new_cached_nonce_review_store(
    inner: Rc<dyn NonceReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn NonceReviewStore> {
    Rc::new(CachedNonceReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<NonceReview>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects nonce reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceReviewQuery {
    pub segment: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl NonceReviewQuery {
    pub fn matches(&self, item: &NonceReview) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[NonceReview]) -> Vec<NonceReview> {
        let matched = collections::filter(items, |candidate: &NonceReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_nonce_reviews(store: Rc<dyn NonceReviewStore>, query: &NonceReviewQuery) -> Vec<NonceReview> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn NonceReviewStore> = new_memory_nonce_review_store();
        let item = new_nonce_review("code-435", "name-385");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
