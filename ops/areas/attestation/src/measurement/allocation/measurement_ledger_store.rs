use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps measurement ledgers by code.
pub trait MeasurementLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MeasurementLedger>;
    fn save(&self, item: &MeasurementLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MeasurementLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMeasurementLedgerStore {
    pub items: RefCell<BTreeMap<String, MeasurementLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl MeasurementLedgerStore for MemoryMeasurementLedgerStore {
    fn find(&self, code: &str) -> Option<MeasurementLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MeasurementLedger) {
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

    fn list(&self) -> Vec<MeasurementLedger> {
        let mut out: Vec<MeasurementLedger> = Vec::new();
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

pub fn new_memory_measurement_ledger_store() -> Rc<dyn MeasurementLedgerStore> {
    Rc::new(MemoryMeasurementLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMeasurementLedgerStore {
    pub inner: Rc<dyn MeasurementLedgerStore>,
    pub cache: RefCell<cache::TtlCache<MeasurementLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MeasurementLedgerStore for CachedMeasurementLedgerStore {
    fn find(&self, code: &str) -> Option<MeasurementLedger> {
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

    fn save(&self, item: &MeasurementLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MeasurementLedger> {
        self.inner.list()
    }
}

pub fn new_cached_measurement_ledger_store(
    inner: Rc<dyn MeasurementLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MeasurementLedgerStore> {
    Rc::new(CachedMeasurementLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MeasurementLedger>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects measurement ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementLedgerQuery {
    pub segment: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MeasurementLedgerQuery {
    pub fn matches(&self, item: &MeasurementLedger) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.ends_at < self.min_ends_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MeasurementLedger]) -> Vec<MeasurementLedger> {
        let matched = collections::filter(items, |candidate: &MeasurementLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_measurement_ledgers(
    store: Rc<dyn MeasurementLedgerStore>,
    query: &MeasurementLedgerQuery,
) -> Vec<MeasurementLedger> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn MeasurementLedgerStore> = new_memory_measurement_ledger_store();
        let item = new_measurement_ledger("code-625", "name-121");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
