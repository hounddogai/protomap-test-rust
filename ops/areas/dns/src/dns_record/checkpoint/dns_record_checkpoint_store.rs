use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dns record checkpoints by code.
pub trait DnsRecordCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DnsRecordCheckpoint>;
    fn save(&self, item: &DnsRecordCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DnsRecordCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDnsRecordCheckpointStore {
    pub items: RefCell<BTreeMap<String, DnsRecordCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl DnsRecordCheckpointStore for MemoryDnsRecordCheckpointStore {
    fn find(&self, code: &str) -> Option<DnsRecordCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DnsRecordCheckpoint) {
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

    fn list(&self) -> Vec<DnsRecordCheckpoint> {
        let mut out: Vec<DnsRecordCheckpoint> = Vec::new();
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

pub fn new_memory_dns_record_checkpoint_store() -> Rc<dyn DnsRecordCheckpointStore> {
    Rc::new(MemoryDnsRecordCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDnsRecordCheckpointStore {
    pub inner: Rc<dyn DnsRecordCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<DnsRecordCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DnsRecordCheckpointStore for CachedDnsRecordCheckpointStore {
    fn find(&self, code: &str) -> Option<DnsRecordCheckpoint> {
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

    fn save(&self, item: &DnsRecordCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DnsRecordCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_dns_record_checkpoint_store(
    inner: Rc<dyn DnsRecordCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DnsRecordCheckpointStore> {
    Rc::new(CachedDnsRecordCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DnsRecordCheckpoint>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dns record checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordCheckpointQuery {
    pub scope: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DnsRecordCheckpointQuery {
    pub fn matches(&self, item: &DnsRecordCheckpoint) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
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

    pub fn apply(&self, items: &[DnsRecordCheckpoint]) -> Vec<DnsRecordCheckpoint> {
        let matched = collections::filter(items, |candidate: &DnsRecordCheckpoint| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dns_record_checkpoints(
    store: Rc<dyn DnsRecordCheckpointStore>,
    query: &DnsRecordCheckpointQuery,
) -> Vec<DnsRecordCheckpoint> {
    query.apply(&store.list())
}
