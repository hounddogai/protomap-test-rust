use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default zone reviews by code.
pub trait DefaultZoneReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultZoneReview>;
    fn save(&self, item: &DefaultZoneReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultZoneReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultZoneReviewStore {
    pub items: RefCell<BTreeMap<String, DefaultZoneReview>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultZoneReviewStore for MemoryDefaultZoneReviewStore {
    fn find(&self, code: &str) -> Option<DefaultZoneReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultZoneReview) {
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

    fn list(&self) -> Vec<DefaultZoneReview> {
        let mut out: Vec<DefaultZoneReview> = Vec::new();
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

pub fn new_memory_default_zone_review_store() -> Rc<dyn DefaultZoneReviewStore> {
    Rc::new(MemoryDefaultZoneReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultZoneReviewStore {
    pub inner: Rc<dyn DefaultZoneReviewStore>,
    pub cache: RefCell<cache::TtlCache<DefaultZoneReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultZoneReviewStore for CachedDefaultZoneReviewStore {
    fn find(&self, code: &str) -> Option<DefaultZoneReview> {
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

    fn save(&self, item: &DefaultZoneReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultZoneReview> {
        self.inner.list()
    }
}

pub fn new_cached_default_zone_review_store(
    inner: Rc<dyn DefaultZoneReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultZoneReviewStore> {
    Rc::new(CachedDefaultZoneReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultZoneReview>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default zone reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneReviewQuery {
    pub name: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultZoneReviewQuery {
    pub fn matches(&self, item: &DefaultZoneReview) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.cost < self.min_cost {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultZoneReview]) -> Vec<DefaultZoneReview> {
        let matched = collections::filter(items, |candidate: &DefaultZoneReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_zone_reviews(
    store: Rc<dyn DefaultZoneReviewStore>,
    query: &DefaultZoneReviewQuery,
) -> Vec<DefaultZoneReview> {
    query.apply(&store.list())
}
