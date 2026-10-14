use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dns record batches by code.
pub trait DnsRecordBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DnsRecordBatch>;
    fn save(&self, item: &DnsRecordBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DnsRecordBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDnsRecordBatchStore {
    pub items: RefCell<BTreeMap<String, DnsRecordBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl DnsRecordBatchStore for MemoryDnsRecordBatchStore {
    fn find(&self, code: &str) -> Option<DnsRecordBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DnsRecordBatch) {
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

    fn list(&self) -> Vec<DnsRecordBatch> {
        let mut out: Vec<DnsRecordBatch> = Vec::new();
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

pub fn new_memory_dns_record_batch_store() -> Rc<dyn DnsRecordBatchStore> {
    Rc::new(MemoryDnsRecordBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDnsRecordBatchStore {
    pub inner: Rc<dyn DnsRecordBatchStore>,
    pub cache: RefCell<cache::TtlCache<DnsRecordBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DnsRecordBatchStore for CachedDnsRecordBatchStore {
    fn find(&self, code: &str) -> Option<DnsRecordBatch> {
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

    fn save(&self, item: &DnsRecordBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DnsRecordBatch> {
        self.inner.list()
    }
}

pub fn new_cached_dns_record_batch_store(
    inner: Rc<dyn DnsRecordBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DnsRecordBatchStore> {
    Rc::new(CachedDnsRecordBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DnsRecordBatch>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dns record batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBatchQuery {
    pub label: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DnsRecordBatchQuery {
    pub fn matches(&self, item: &DnsRecordBatch) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DnsRecordBatch]) -> Vec<DnsRecordBatch> {
        let matched = collections::filter(items, |candidate: &DnsRecordBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dns_record_batches(store: Rc<dyn DnsRecordBatchStore>, query: &DnsRecordBatchQuery) -> Vec<DnsRecordBatch> {
    query.apply(&store.list())
}
