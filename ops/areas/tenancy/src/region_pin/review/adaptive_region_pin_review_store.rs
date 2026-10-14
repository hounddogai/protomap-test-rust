use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive region pin reviews by code.
pub trait AdaptiveRegionPinReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveRegionPinReview>;
    fn save(&self, item: &AdaptiveRegionPinReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveRegionPinReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveRegionPinReviewStore {
    pub items: RefCell<BTreeMap<String, AdaptiveRegionPinReview>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveRegionPinReviewStore for MemoryAdaptiveRegionPinReviewStore {
    fn find(&self, code: &str) -> Option<AdaptiveRegionPinReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveRegionPinReview) {
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

    fn list(&self) -> Vec<AdaptiveRegionPinReview> {
        let mut out: Vec<AdaptiveRegionPinReview> = Vec::new();
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

pub fn new_memory_adaptive_region_pin_review_store() -> Rc<dyn AdaptiveRegionPinReviewStore> {
    Rc::new(MemoryAdaptiveRegionPinReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveRegionPinReviewStore {
    pub inner: Rc<dyn AdaptiveRegionPinReviewStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveRegionPinReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveRegionPinReviewStore for CachedAdaptiveRegionPinReviewStore {
    fn find(&self, code: &str) -> Option<AdaptiveRegionPinReview> {
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

    fn save(&self, item: &AdaptiveRegionPinReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveRegionPinReview> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_region_pin_review_store(
    inner: Rc<dyn AdaptiveRegionPinReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveRegionPinReviewStore> {
    Rc::new(CachedAdaptiveRegionPinReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveRegionPinReview>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive region pin reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinReviewQuery {
    pub label: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveRegionPinReviewQuery {
    pub fn matches(&self, item: &AdaptiveRegionPinReview) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveRegionPinReview]) -> Vec<AdaptiveRegionPinReview> {
        let matched = collections::filter(items, |candidate: &AdaptiveRegionPinReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_region_pin_reviews(
    store: Rc<dyn AdaptiveRegionPinReviewStore>,
    query: &AdaptiveRegionPinReviewQuery,
) -> Vec<AdaptiveRegionPinReview> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AdaptiveRegionPinReviewStore> = new_memory_adaptive_region_pin_review_store();
        let item = new_adaptive_region_pin_review("code-296", "label-457");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
