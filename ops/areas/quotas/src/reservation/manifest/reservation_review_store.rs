use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps reservation reviews by code.
pub trait ReservationReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ReservationReview>;
    fn save(&self, item: &ReservationReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ReservationReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryReservationReviewStore {
    pub items: RefCell<BTreeMap<String, ReservationReview>>,
    pub order: RefCell<Vec<String>>,
}

impl ReservationReviewStore for MemoryReservationReviewStore {
    fn find(&self, code: &str) -> Option<ReservationReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ReservationReview) {
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

    fn list(&self) -> Vec<ReservationReview> {
        let mut out: Vec<ReservationReview> = Vec::new();
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

pub fn new_memory_reservation_review_store() -> Rc<dyn ReservationReviewStore> {
    Rc::new(MemoryReservationReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedReservationReviewStore {
    pub inner: Rc<dyn ReservationReviewStore>,
    pub cache: RefCell<cache::TtlCache<ReservationReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ReservationReviewStore for CachedReservationReviewStore {
    fn find(&self, code: &str) -> Option<ReservationReview> {
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

    fn save(&self, item: &ReservationReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ReservationReview> {
        self.inner.list()
    }
}

pub fn new_cached_reservation_review_store(
    inner: Rc<dyn ReservationReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ReservationReviewStore> {
    Rc::new(CachedReservationReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ReservationReview>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects reservation reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationReviewQuery {
    pub region: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ReservationReviewQuery {
    pub fn matches(&self, item: &ReservationReview) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ReservationReview]) -> Vec<ReservationReview> {
        let matched = collections::filter(items, |candidate: &ReservationReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_reservation_reviews(
    store: Rc<dyn ReservationReviewStore>,
    query: &ReservationReviewQuery,
) -> Vec<ReservationReview> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ReservationReviewStore> = new_memory_reservation_review_store();
        let item = new_reservation_review("code-324", "name-448");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
