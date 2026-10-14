use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps reservation quotas by code.
pub trait ReservationQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ReservationQuota>;
    fn save(&self, item: &ReservationQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ReservationQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryReservationQuotaStore {
    pub items: RefCell<BTreeMap<String, ReservationQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl ReservationQuotaStore for MemoryReservationQuotaStore {
    fn find(&self, code: &str) -> Option<ReservationQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ReservationQuota) {
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

    fn list(&self) -> Vec<ReservationQuota> {
        let mut out: Vec<ReservationQuota> = Vec::new();
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

pub fn new_memory_reservation_quota_store() -> Rc<dyn ReservationQuotaStore> {
    Rc::new(MemoryReservationQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedReservationQuotaStore {
    pub inner: Rc<dyn ReservationQuotaStore>,
    pub cache: RefCell<cache::TtlCache<ReservationQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ReservationQuotaStore for CachedReservationQuotaStore {
    fn find(&self, code: &str) -> Option<ReservationQuota> {
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

    fn save(&self, item: &ReservationQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ReservationQuota> {
        self.inner.list()
    }
}

pub fn new_cached_reservation_quota_store(
    inner: Rc<dyn ReservationQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ReservationQuotaStore> {
    Rc::new(CachedReservationQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ReservationQuota>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects reservation quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationQuotaQuery {
    pub label: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ReservationQuotaQuery {
    pub fn matches(&self, item: &ReservationQuota) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.ceiling < self.min_ceiling {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ReservationQuota]) -> Vec<ReservationQuota> {
        let matched = collections::filter(items, |candidate: &ReservationQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_reservation_quotas(
    store: Rc<dyn ReservationQuotaStore>,
    query: &ReservationQuotaQuery,
) -> Vec<ReservationQuota> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ReservationQuotaStore> = new_memory_reservation_quota_store();
        let item = new_reservation_quota("code-715", "label-242");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
