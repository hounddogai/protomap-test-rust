use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional reservation exceptions by code.
pub trait RegionalReservationExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalReservationException>;
    fn save(&self, item: &RegionalReservationException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalReservationException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalReservationExceptionStore {
    pub items: RefCell<BTreeMap<String, RegionalReservationException>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalReservationExceptionStore for MemoryRegionalReservationExceptionStore {
    fn find(&self, code: &str) -> Option<RegionalReservationException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalReservationException) {
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

    fn list(&self) -> Vec<RegionalReservationException> {
        let mut out: Vec<RegionalReservationException> = Vec::new();
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

pub fn new_memory_regional_reservation_exception_store() -> Rc<dyn RegionalReservationExceptionStore> {
    Rc::new(MemoryRegionalReservationExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalReservationExceptionStore {
    pub inner: Rc<dyn RegionalReservationExceptionStore>,
    pub cache: RefCell<cache::TtlCache<RegionalReservationException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalReservationExceptionStore for CachedRegionalReservationExceptionStore {
    fn find(&self, code: &str) -> Option<RegionalReservationException> {
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

    fn save(&self, item: &RegionalReservationException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalReservationException> {
        self.inner.list()
    }
}

pub fn new_cached_regional_reservation_exception_store(
    inner: Rc<dyn RegionalReservationExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalReservationExceptionStore> {
    Rc::new(CachedRegionalReservationExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalReservationException>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional reservation exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationExceptionQuery {
    pub label: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalReservationExceptionQuery {
    pub fn matches(&self, item: &RegionalReservationException) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalReservationException]) -> Vec<RegionalReservationException> {
        let matched = collections::filter(items, |candidate: &RegionalReservationException| {
            self.matches(candidate)
        });
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_reservation_exceptions(
    store: Rc<dyn RegionalReservationExceptionStore>,
    query: &RegionalReservationExceptionQuery,
) -> Vec<RegionalReservationException> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalReservationExceptionStore> = new_memory_regional_reservation_exception_store();
        let item = new_regional_reservation_exception("code-346", "label-899");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
