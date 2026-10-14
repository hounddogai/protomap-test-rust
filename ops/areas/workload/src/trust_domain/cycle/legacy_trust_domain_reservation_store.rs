use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy trust domain reservations by code.
pub trait LegacyTrustDomainReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyTrustDomainReservation>;
    fn save(&self, item: &LegacyTrustDomainReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyTrustDomainReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyTrustDomainReservationStore {
    pub items: RefCell<BTreeMap<String, LegacyTrustDomainReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyTrustDomainReservationStore for MemoryLegacyTrustDomainReservationStore {
    fn find(&self, code: &str) -> Option<LegacyTrustDomainReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyTrustDomainReservation) {
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

    fn list(&self) -> Vec<LegacyTrustDomainReservation> {
        let mut out: Vec<LegacyTrustDomainReservation> = Vec::new();
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

pub fn new_memory_legacy_trust_domain_reservation_store() -> Rc<dyn LegacyTrustDomainReservationStore> {
    Rc::new(MemoryLegacyTrustDomainReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyTrustDomainReservationStore {
    pub inner: Rc<dyn LegacyTrustDomainReservationStore>,
    pub cache: RefCell<cache::TtlCache<LegacyTrustDomainReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyTrustDomainReservationStore for CachedLegacyTrustDomainReservationStore {
    fn find(&self, code: &str) -> Option<LegacyTrustDomainReservation> {
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

    fn save(&self, item: &LegacyTrustDomainReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyTrustDomainReservation> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_trust_domain_reservation_store(
    inner: Rc<dyn LegacyTrustDomainReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyTrustDomainReservationStore> {
    Rc::new(CachedLegacyTrustDomainReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyTrustDomainReservation>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy trust domain reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainReservationQuery {
    pub title: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyTrustDomainReservationQuery {
    pub fn matches(&self, item: &LegacyTrustDomainReservation) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[LegacyTrustDomainReservation]) -> Vec<LegacyTrustDomainReservation> {
        let matched = collections::filter(items, |candidate: &LegacyTrustDomainReservation| {
            self.matches(candidate)
        });
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_trust_domain_reservations(
    store: Rc<dyn LegacyTrustDomainReservationStore>,
    query: &LegacyTrustDomainReservationQuery,
) -> Vec<LegacyTrustDomainReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyTrustDomainReservationStore> = new_memory_legacy_trust_domain_reservation_store();
        let item = new_legacy_trust_domain_reservation("code-118", "title-825");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
