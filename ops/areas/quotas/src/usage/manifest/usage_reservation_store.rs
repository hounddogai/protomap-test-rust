use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps usage reservations by code.
pub trait UsageReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<UsageReservation>;
    fn save(&self, item: &UsageReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<UsageReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryUsageReservationStore {
    pub items: RefCell<BTreeMap<String, UsageReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl UsageReservationStore for MemoryUsageReservationStore {
    fn find(&self, code: &str) -> Option<UsageReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &UsageReservation) {
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

    fn list(&self) -> Vec<UsageReservation> {
        let mut out: Vec<UsageReservation> = Vec::new();
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

pub fn new_memory_usage_reservation_store() -> Rc<dyn UsageReservationStore> {
    Rc::new(MemoryUsageReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedUsageReservationStore {
    pub inner: Rc<dyn UsageReservationStore>,
    pub cache: RefCell<cache::TtlCache<UsageReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl UsageReservationStore for CachedUsageReservationStore {
    fn find(&self, code: &str) -> Option<UsageReservation> {
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

    fn save(&self, item: &UsageReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<UsageReservation> {
        self.inner.list()
    }
}

pub fn new_cached_usage_reservation_store(
    inner: Rc<dyn UsageReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn UsageReservationStore> {
    Rc::new(CachedUsageReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<UsageReservation>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects usage reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageReservationQuery {
    pub scope: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl UsageReservationQuery {
    pub fn matches(&self, item: &UsageReservation) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[UsageReservation]) -> Vec<UsageReservation> {
        let matched = collections::filter(items, |candidate: &UsageReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_usage_reservations(
    store: Rc<dyn UsageReservationStore>,
    query: &UsageReservationQuery,
) -> Vec<UsageReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn UsageReservationStore> = new_memory_usage_reservation_store();
        let item = new_usage_reservation("code-554", "title-643");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
