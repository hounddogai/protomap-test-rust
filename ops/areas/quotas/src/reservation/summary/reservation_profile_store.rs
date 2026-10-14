use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps reservation profiles by code.
pub trait ReservationProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ReservationProfile>;
    fn save(&self, item: &ReservationProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ReservationProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryReservationProfileStore {
    pub items: RefCell<BTreeMap<String, ReservationProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl ReservationProfileStore for MemoryReservationProfileStore {
    fn find(&self, code: &str) -> Option<ReservationProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ReservationProfile) {
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

    fn list(&self) -> Vec<ReservationProfile> {
        let mut out: Vec<ReservationProfile> = Vec::new();
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

pub fn new_memory_reservation_profile_store() -> Rc<dyn ReservationProfileStore> {
    Rc::new(MemoryReservationProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedReservationProfileStore {
    pub inner: Rc<dyn ReservationProfileStore>,
    pub cache: RefCell<cache::TtlCache<ReservationProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ReservationProfileStore for CachedReservationProfileStore {
    fn find(&self, code: &str) -> Option<ReservationProfile> {
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

    fn save(&self, item: &ReservationProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ReservationProfile> {
        self.inner.list()
    }
}

pub fn new_cached_reservation_profile_store(
    inner: Rc<dyn ReservationProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ReservationProfileStore> {
    Rc::new(CachedReservationProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ReservationProfile>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects reservation profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationProfileQuery {
    pub title: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ReservationProfileQuery {
    pub fn matches(&self, item: &ReservationProfile) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ReservationProfile]) -> Vec<ReservationProfile> {
        let matched = collections::filter(items, |candidate: &ReservationProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_reservation_profiles(
    store: Rc<dyn ReservationProfileStore>,
    query: &ReservationProfileQuery,
) -> Vec<ReservationProfile> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ReservationProfileStore> = new_memory_reservation_profile_store();
        let item = new_reservation_profile("code-676", "title-837");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
