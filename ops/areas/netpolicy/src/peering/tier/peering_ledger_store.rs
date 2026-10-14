use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps peering ledgers by code.
pub trait PeeringLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PeeringLedger>;
    fn save(&self, item: &PeeringLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PeeringLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPeeringLedgerStore {
    pub items: RefCell<BTreeMap<String, PeeringLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl PeeringLedgerStore for MemoryPeeringLedgerStore {
    fn find(&self, code: &str) -> Option<PeeringLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PeeringLedger) {
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

    fn list(&self) -> Vec<PeeringLedger> {
        let mut out: Vec<PeeringLedger> = Vec::new();
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

pub fn new_memory_peering_ledger_store() -> Rc<dyn PeeringLedgerStore> {
    Rc::new(MemoryPeeringLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPeeringLedgerStore {
    pub inner: Rc<dyn PeeringLedgerStore>,
    pub cache: RefCell<cache::TtlCache<PeeringLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PeeringLedgerStore for CachedPeeringLedgerStore {
    fn find(&self, code: &str) -> Option<PeeringLedger> {
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

    fn save(&self, item: &PeeringLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PeeringLedger> {
        self.inner.list()
    }
}

pub fn new_cached_peering_ledger_store(
    inner: Rc<dyn PeeringLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PeeringLedgerStore> {
    Rc::new(CachedPeeringLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PeeringLedger>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects peering ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringLedgerQuery {
    pub namespace: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PeeringLedgerQuery {
    pub fn matches(&self, item: &PeeringLedger) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PeeringLedger]) -> Vec<PeeringLedger> {
        let matched = collections::filter(items, |candidate: &PeeringLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_peering_ledgers(store: Rc<dyn PeeringLedgerStore>, query: &PeeringLedgerQuery) -> Vec<PeeringLedger> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PeeringLedgerStore> = new_memory_peering_ledger_store();
        let item = new_peering_ledger("code-859", "label-2");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
