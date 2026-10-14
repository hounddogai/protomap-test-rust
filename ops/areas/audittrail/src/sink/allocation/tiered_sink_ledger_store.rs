use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered sink ledgers by code.
pub trait TieredSinkLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredSinkLedger>;
    fn save(&self, item: &TieredSinkLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredSinkLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredSinkLedgerStore {
    pub items: RefCell<BTreeMap<String, TieredSinkLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredSinkLedgerStore for MemoryTieredSinkLedgerStore {
    fn find(&self, code: &str) -> Option<TieredSinkLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredSinkLedger) {
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

    fn list(&self) -> Vec<TieredSinkLedger> {
        let mut out: Vec<TieredSinkLedger> = Vec::new();
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

pub fn new_memory_tiered_sink_ledger_store() -> Rc<dyn TieredSinkLedgerStore> {
    Rc::new(MemoryTieredSinkLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredSinkLedgerStore {
    pub inner: Rc<dyn TieredSinkLedgerStore>,
    pub cache: RefCell<cache::TtlCache<TieredSinkLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredSinkLedgerStore for CachedTieredSinkLedgerStore {
    fn find(&self, code: &str) -> Option<TieredSinkLedger> {
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

    fn save(&self, item: &TieredSinkLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredSinkLedger> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_sink_ledger_store(
    inner: Rc<dyn TieredSinkLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredSinkLedgerStore> {
    Rc::new(CachedTieredSinkLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredSinkLedger>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered sink ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkLedgerQuery {
    pub region: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredSinkLedgerQuery {
    pub fn matches(&self, item: &TieredSinkLedger) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredSinkLedger]) -> Vec<TieredSinkLedger> {
        let matched = collections::filter(items, |candidate: &TieredSinkLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_sink_ledgers(
    store: Rc<dyn TieredSinkLedgerStore>,
    query: &TieredSinkLedgerQuery,
) -> Vec<TieredSinkLedger> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredSinkLedgerStore> = new_memory_tiered_sink_ledger_store();
        let item = new_tiered_sink_ledger("code-588", "title-190");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
