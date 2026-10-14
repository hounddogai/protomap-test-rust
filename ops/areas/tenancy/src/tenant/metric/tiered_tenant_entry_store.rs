use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered tenant entries by code.
pub trait TieredTenantEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredTenantEntry>;
    fn save(&self, item: &TieredTenantEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredTenantEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredTenantEntryStore {
    pub items: RefCell<BTreeMap<String, TieredTenantEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredTenantEntryStore for MemoryTieredTenantEntryStore {
    fn find(&self, code: &str) -> Option<TieredTenantEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredTenantEntry) {
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

    fn list(&self) -> Vec<TieredTenantEntry> {
        let mut out: Vec<TieredTenantEntry> = Vec::new();
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

pub fn new_memory_tiered_tenant_entry_store() -> Rc<dyn TieredTenantEntryStore> {
    Rc::new(MemoryTieredTenantEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredTenantEntryStore {
    pub inner: Rc<dyn TieredTenantEntryStore>,
    pub cache: RefCell<cache::TtlCache<TieredTenantEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredTenantEntryStore for CachedTieredTenantEntryStore {
    fn find(&self, code: &str) -> Option<TieredTenantEntry> {
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

    fn save(&self, item: &TieredTenantEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredTenantEntry> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_tenant_entry_store(
    inner: Rc<dyn TieredTenantEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredTenantEntryStore> {
    Rc::new(CachedTieredTenantEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredTenantEntry>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered tenant entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantEntryQuery {
    pub region: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredTenantEntryQuery {
    pub fn matches(&self, item: &TieredTenantEntry) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[TieredTenantEntry]) -> Vec<TieredTenantEntry> {
        let matched = collections::filter(items, |candidate: &TieredTenantEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_tenant_entries(
    store: Rc<dyn TieredTenantEntryStore>,
    query: &TieredTenantEntryQuery,
) -> Vec<TieredTenantEntry> {
    query.apply(&store.list())
}
