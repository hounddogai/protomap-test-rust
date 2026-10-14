use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps page reservations by code.
pub trait PageReservationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PageReservation>;
    fn save(&self, item: &PageReservation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PageReservation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPageReservationStore {
    pub items: RefCell<BTreeMap<String, PageReservation>>,
    pub order: RefCell<Vec<String>>,
}

impl PageReservationStore for MemoryPageReservationStore {
    fn find(&self, code: &str) -> Option<PageReservation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PageReservation) {
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

    fn list(&self) -> Vec<PageReservation> {
        let mut out: Vec<PageReservation> = Vec::new();
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

pub fn new_memory_page_reservation_store() -> Rc<dyn PageReservationStore> {
    Rc::new(MemoryPageReservationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPageReservationStore {
    pub inner: Rc<dyn PageReservationStore>,
    pub cache: RefCell<cache::TtlCache<PageReservation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PageReservationStore for CachedPageReservationStore {
    fn find(&self, code: &str) -> Option<PageReservation> {
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

    fn save(&self, item: &PageReservation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PageReservation> {
        self.inner.list()
    }
}

pub fn new_cached_page_reservation_store(
    inner: Rc<dyn PageReservationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PageReservationStore> {
    Rc::new(CachedPageReservationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PageReservation>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects page reservations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PageReservationQuery {
    pub title: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PageReservationQuery {
    pub fn matches(&self, item: &PageReservation) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PageReservation]) -> Vec<PageReservation> {
        let matched = collections::filter(items, |candidate: &PageReservation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_page_reservations(
    store: Rc<dyn PageReservationStore>,
    query: &PageReservationQuery,
) -> Vec<PageReservation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PageReservationStore> = new_memory_page_reservation_store();
        let item = new_page_reservation("code-944", "title-924");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
