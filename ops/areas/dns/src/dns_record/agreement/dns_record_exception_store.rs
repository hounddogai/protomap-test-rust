use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dns record exceptions by code.
pub trait DnsRecordExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DnsRecordException>;
    fn save(&self, item: &DnsRecordException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DnsRecordException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDnsRecordExceptionStore {
    pub items: RefCell<BTreeMap<String, DnsRecordException>>,
    pub order: RefCell<Vec<String>>,
}

impl DnsRecordExceptionStore for MemoryDnsRecordExceptionStore {
    fn find(&self, code: &str) -> Option<DnsRecordException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DnsRecordException) {
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

    fn list(&self) -> Vec<DnsRecordException> {
        let mut out: Vec<DnsRecordException> = Vec::new();
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

pub fn new_memory_dns_record_exception_store() -> Rc<dyn DnsRecordExceptionStore> {
    Rc::new(MemoryDnsRecordExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDnsRecordExceptionStore {
    pub inner: Rc<dyn DnsRecordExceptionStore>,
    pub cache: RefCell<cache::TtlCache<DnsRecordException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DnsRecordExceptionStore for CachedDnsRecordExceptionStore {
    fn find(&self, code: &str) -> Option<DnsRecordException> {
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

    fn save(&self, item: &DnsRecordException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DnsRecordException> {
        self.inner.list()
    }
}

pub fn new_cached_dns_record_exception_store(
    inner: Rc<dyn DnsRecordExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DnsRecordExceptionStore> {
    Rc::new(CachedDnsRecordExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DnsRecordException>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dns record exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordExceptionQuery {
    pub segment: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DnsRecordExceptionQuery {
    pub fn matches(&self, item: &DnsRecordException) -> bool {
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

    pub fn apply(&self, items: &[DnsRecordException]) -> Vec<DnsRecordException> {
        let matched = collections::filter(items, |candidate: &DnsRecordException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dns_record_exceptions(
    store: Rc<dyn DnsRecordExceptionStore>,
    query: &DnsRecordExceptionQuery,
) -> Vec<DnsRecordException> {
    query.apply(&store.list())
}
