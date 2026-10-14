use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped budget burn entries by code.
pub trait CappedBudgetBurnEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedBudgetBurnEntry>;
    fn save(&self, item: &CappedBudgetBurnEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedBudgetBurnEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedBudgetBurnEntryStore {
    pub items: RefCell<BTreeMap<String, CappedBudgetBurnEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedBudgetBurnEntryStore for MemoryCappedBudgetBurnEntryStore {
    fn find(&self, code: &str) -> Option<CappedBudgetBurnEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedBudgetBurnEntry) {
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

    fn list(&self) -> Vec<CappedBudgetBurnEntry> {
        let mut out: Vec<CappedBudgetBurnEntry> = Vec::new();
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

pub fn new_memory_capped_budget_burn_entry_store() -> Rc<dyn CappedBudgetBurnEntryStore> {
    Rc::new(MemoryCappedBudgetBurnEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedBudgetBurnEntryStore {
    pub inner: Rc<dyn CappedBudgetBurnEntryStore>,
    pub cache: RefCell<cache::TtlCache<CappedBudgetBurnEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedBudgetBurnEntryStore for CachedCappedBudgetBurnEntryStore {
    fn find(&self, code: &str) -> Option<CappedBudgetBurnEntry> {
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

    fn save(&self, item: &CappedBudgetBurnEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedBudgetBurnEntry> {
        self.inner.list()
    }
}

pub fn new_cached_capped_budget_burn_entry_store(
    inner: Rc<dyn CappedBudgetBurnEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedBudgetBurnEntryStore> {
    Rc::new(CachedCappedBudgetBurnEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedBudgetBurnEntry>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped budget burn entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnEntryQuery {
    pub namespace: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedBudgetBurnEntryQuery {
    pub fn matches(&self, item: &CappedBudgetBurnEntry) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedBudgetBurnEntry]) -> Vec<CappedBudgetBurnEntry> {
        let matched = collections::filter(items, |candidate: &CappedBudgetBurnEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_budget_burn_entries(
    store: Rc<dyn CappedBudgetBurnEntryStore>,
    query: &CappedBudgetBurnEntryQuery,
) -> Vec<CappedBudgetBurnEntry> {
    query.apply(&store.list())
}
