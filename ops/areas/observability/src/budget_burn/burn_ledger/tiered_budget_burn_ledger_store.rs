use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered budget burn ledgers by code.
pub trait TieredBudgetBurnLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredBudgetBurnLedger>;
    fn save(&self, item: &TieredBudgetBurnLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredBudgetBurnLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredBudgetBurnLedgerStore {
    pub items: RefCell<BTreeMap<String, TieredBudgetBurnLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredBudgetBurnLedgerStore for MemoryTieredBudgetBurnLedgerStore {
    fn find(&self, code: &str) -> Option<TieredBudgetBurnLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredBudgetBurnLedger) {
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

    fn list(&self) -> Vec<TieredBudgetBurnLedger> {
        let mut out: Vec<TieredBudgetBurnLedger> = Vec::new();
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

pub fn new_memory_tiered_budget_burn_ledger_store() -> Rc<dyn TieredBudgetBurnLedgerStore> {
    Rc::new(MemoryTieredBudgetBurnLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredBudgetBurnLedgerStore {
    pub inner: Rc<dyn TieredBudgetBurnLedgerStore>,
    pub cache: RefCell<cache::TtlCache<TieredBudgetBurnLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredBudgetBurnLedgerStore for CachedTieredBudgetBurnLedgerStore {
    fn find(&self, code: &str) -> Option<TieredBudgetBurnLedger> {
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

    fn save(&self, item: &TieredBudgetBurnLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredBudgetBurnLedger> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_budget_burn_ledger_store(
    inner: Rc<dyn TieredBudgetBurnLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredBudgetBurnLedgerStore> {
    Rc::new(CachedTieredBudgetBurnLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredBudgetBurnLedger>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered budget burn ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnLedgerQuery {
    pub locale: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredBudgetBurnLedgerQuery {
    pub fn matches(&self, item: &TieredBudgetBurnLedger) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredBudgetBurnLedger]) -> Vec<TieredBudgetBurnLedger> {
        let matched = collections::filter(items, |candidate: &TieredBudgetBurnLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_budget_burn_ledgers(
    store: Rc<dyn TieredBudgetBurnLedgerStore>,
    query: &TieredBudgetBurnLedgerQuery,
) -> Vec<TieredBudgetBurnLedger> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredBudgetBurnLedgerStore> = new_memory_tiered_budget_burn_ledger_store();
        let item = new_tiered_budget_burn_ledger("code-511", "title-499");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
