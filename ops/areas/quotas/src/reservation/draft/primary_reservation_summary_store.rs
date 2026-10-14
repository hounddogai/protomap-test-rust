use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary reservation summaries by code.
pub trait PrimaryReservationSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryReservationSummary>;
    fn save(&self, item: &PrimaryReservationSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryReservationSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryReservationSummaryStore {
    pub items: RefCell<BTreeMap<String, PrimaryReservationSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryReservationSummaryStore for MemoryPrimaryReservationSummaryStore {
    fn find(&self, code: &str) -> Option<PrimaryReservationSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryReservationSummary) {
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

    fn list(&self) -> Vec<PrimaryReservationSummary> {
        let mut out: Vec<PrimaryReservationSummary> = Vec::new();
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

pub fn new_memory_primary_reservation_summary_store() -> Rc<dyn PrimaryReservationSummaryStore> {
    Rc::new(MemoryPrimaryReservationSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryReservationSummaryStore {
    pub inner: Rc<dyn PrimaryReservationSummaryStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryReservationSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryReservationSummaryStore for CachedPrimaryReservationSummaryStore {
    fn find(&self, code: &str) -> Option<PrimaryReservationSummary> {
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

    fn save(&self, item: &PrimaryReservationSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryReservationSummary> {
        self.inner.list()
    }
}

pub fn new_cached_primary_reservation_summary_store(
    inner: Rc<dyn PrimaryReservationSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryReservationSummaryStore> {
    Rc::new(CachedPrimaryReservationSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryReservationSummary>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary reservation summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationSummaryQuery {
    pub label: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryReservationSummaryQuery {
    pub fn matches(&self, item: &PrimaryReservationSummary) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[PrimaryReservationSummary]) -> Vec<PrimaryReservationSummary> {
        let matched = collections::filter(items, |candidate: &PrimaryReservationSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_reservation_summaries(
    store: Rc<dyn PrimaryReservationSummaryStore>,
    query: &PrimaryReservationSummaryQuery,
) -> Vec<PrimaryReservationSummary> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryReservationSummaryStore> = new_memory_primary_reservation_summary_store();
        let item = new_primary_reservation_summary("code-88", "label-63");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
