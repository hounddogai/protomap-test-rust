use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary dns record summaries by code.
pub trait PrimaryDnsRecordSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryDnsRecordSummary>;
    fn save(&self, item: &PrimaryDnsRecordSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryDnsRecordSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryDnsRecordSummaryStore {
    pub items: RefCell<BTreeMap<String, PrimaryDnsRecordSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryDnsRecordSummaryStore for MemoryPrimaryDnsRecordSummaryStore {
    fn find(&self, code: &str) -> Option<PrimaryDnsRecordSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryDnsRecordSummary) {
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

    fn list(&self) -> Vec<PrimaryDnsRecordSummary> {
        let mut out: Vec<PrimaryDnsRecordSummary> = Vec::new();
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

pub fn new_memory_primary_dns_record_summary_store() -> Rc<dyn PrimaryDnsRecordSummaryStore> {
    Rc::new(MemoryPrimaryDnsRecordSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryDnsRecordSummaryStore {
    pub inner: Rc<dyn PrimaryDnsRecordSummaryStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryDnsRecordSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryDnsRecordSummaryStore for CachedPrimaryDnsRecordSummaryStore {
    fn find(&self, code: &str) -> Option<PrimaryDnsRecordSummary> {
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

    fn save(&self, item: &PrimaryDnsRecordSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryDnsRecordSummary> {
        self.inner.list()
    }
}

pub fn new_cached_primary_dns_record_summary_store(
    inner: Rc<dyn PrimaryDnsRecordSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryDnsRecordSummaryStore> {
    Rc::new(CachedPrimaryDnsRecordSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryDnsRecordSummary>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary dns record summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordSummaryQuery {
    pub label: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryDnsRecordSummaryQuery {
    pub fn matches(&self, item: &PrimaryDnsRecordSummary) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[PrimaryDnsRecordSummary]) -> Vec<PrimaryDnsRecordSummary> {
        let matched = collections::filter(items, |candidate: &PrimaryDnsRecordSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_dns_record_summaries(
    store: Rc<dyn PrimaryDnsRecordSummaryStore>,
    query: &PrimaryDnsRecordSummaryQuery,
) -> Vec<PrimaryDnsRecordSummary> {
    query.apply(&store.list())
}
