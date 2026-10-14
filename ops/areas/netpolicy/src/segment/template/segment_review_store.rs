use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps segment reviews by code.
pub trait SegmentReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SegmentReview>;
    fn save(&self, item: &SegmentReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SegmentReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySegmentReviewStore {
    pub items: RefCell<BTreeMap<String, SegmentReview>>,
    pub order: RefCell<Vec<String>>,
}

impl SegmentReviewStore for MemorySegmentReviewStore {
    fn find(&self, code: &str) -> Option<SegmentReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SegmentReview) {
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

    fn list(&self) -> Vec<SegmentReview> {
        let mut out: Vec<SegmentReview> = Vec::new();
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

pub fn new_memory_segment_review_store() -> Rc<dyn SegmentReviewStore> {
    Rc::new(MemorySegmentReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSegmentReviewStore {
    pub inner: Rc<dyn SegmentReviewStore>,
    pub cache: RefCell<cache::TtlCache<SegmentReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SegmentReviewStore for CachedSegmentReviewStore {
    fn find(&self, code: &str) -> Option<SegmentReview> {
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

    fn save(&self, item: &SegmentReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SegmentReview> {
        self.inner.list()
    }
}

pub fn new_cached_segment_review_store(
    inner: Rc<dyn SegmentReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SegmentReviewStore> {
    Rc::new(CachedSegmentReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SegmentReview>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects segment reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentReviewQuery {
    pub locale: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SegmentReviewQuery {
    pub fn matches(&self, item: &SegmentReview) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SegmentReview]) -> Vec<SegmentReview> {
        let matched = collections::filter(items, |candidate: &SegmentReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_segment_reviews(store: Rc<dyn SegmentReviewStore>, query: &SegmentReviewQuery) -> Vec<SegmentReview> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SegmentReviewStore> = new_memory_segment_review_store();
        let item = new_segment_review("code-924", "name-823");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
