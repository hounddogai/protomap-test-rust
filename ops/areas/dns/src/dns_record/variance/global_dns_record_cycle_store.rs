use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global dns record cycles by code.
pub trait GlobalDnsRecordCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalDnsRecordCycle>;
    fn save(&self, item: &GlobalDnsRecordCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalDnsRecordCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalDnsRecordCycleStore {
    pub items: RefCell<BTreeMap<String, GlobalDnsRecordCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalDnsRecordCycleStore for MemoryGlobalDnsRecordCycleStore {
    fn find(&self, code: &str) -> Option<GlobalDnsRecordCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalDnsRecordCycle) {
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

    fn list(&self) -> Vec<GlobalDnsRecordCycle> {
        let mut out: Vec<GlobalDnsRecordCycle> = Vec::new();
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

pub fn new_memory_global_dns_record_cycle_store() -> Rc<dyn GlobalDnsRecordCycleStore> {
    Rc::new(MemoryGlobalDnsRecordCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalDnsRecordCycleStore {
    pub inner: Rc<dyn GlobalDnsRecordCycleStore>,
    pub cache: RefCell<cache::TtlCache<GlobalDnsRecordCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalDnsRecordCycleStore for CachedGlobalDnsRecordCycleStore {
    fn find(&self, code: &str) -> Option<GlobalDnsRecordCycle> {
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

    fn save(&self, item: &GlobalDnsRecordCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalDnsRecordCycle> {
        self.inner.list()
    }
}

pub fn new_cached_global_dns_record_cycle_store(
    inner: Rc<dyn GlobalDnsRecordCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalDnsRecordCycleStore> {
    Rc::new(CachedGlobalDnsRecordCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalDnsRecordCycle>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global dns record cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDnsRecordCycleQuery {
    pub source: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalDnsRecordCycleQuery {
    pub fn matches(&self, item: &GlobalDnsRecordCycle) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalDnsRecordCycle]) -> Vec<GlobalDnsRecordCycle> {
        let matched = collections::filter(items, |candidate: &GlobalDnsRecordCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_dns_record_cycles(
    store: Rc<dyn GlobalDnsRecordCycleStore>,
    query: &GlobalDnsRecordCycleQuery,
) -> Vec<GlobalDnsRecordCycle> {
    query.apply(&store.list())
}
