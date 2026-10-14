use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dns record adjustments by code.
pub trait DnsRecordAdjustmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DnsRecordAdjustment>;
    fn save(&self, item: &DnsRecordAdjustment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DnsRecordAdjustment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDnsRecordAdjustmentStore {
    pub items: RefCell<BTreeMap<String, DnsRecordAdjustment>>,
    pub order: RefCell<Vec<String>>,
}

impl DnsRecordAdjustmentStore for MemoryDnsRecordAdjustmentStore {
    fn find(&self, code: &str) -> Option<DnsRecordAdjustment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DnsRecordAdjustment) {
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

    fn list(&self) -> Vec<DnsRecordAdjustment> {
        let mut out: Vec<DnsRecordAdjustment> = Vec::new();
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

pub fn new_memory_dns_record_adjustment_store() -> Rc<dyn DnsRecordAdjustmentStore> {
    Rc::new(MemoryDnsRecordAdjustmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDnsRecordAdjustmentStore {
    pub inner: Rc<dyn DnsRecordAdjustmentStore>,
    pub cache: RefCell<cache::TtlCache<DnsRecordAdjustment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DnsRecordAdjustmentStore for CachedDnsRecordAdjustmentStore {
    fn find(&self, code: &str) -> Option<DnsRecordAdjustment> {
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

    fn save(&self, item: &DnsRecordAdjustment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DnsRecordAdjustment> {
        self.inner.list()
    }
}

pub fn new_cached_dns_record_adjustment_store(
    inner: Rc<dyn DnsRecordAdjustmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DnsRecordAdjustmentStore> {
    Rc::new(CachedDnsRecordAdjustmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DnsRecordAdjustment>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dns record adjustments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAdjustmentQuery {
    pub unit: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DnsRecordAdjustmentQuery {
    pub fn matches(&self, item: &DnsRecordAdjustment) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DnsRecordAdjustment]) -> Vec<DnsRecordAdjustment> {
        let matched = collections::filter(items, |candidate: &DnsRecordAdjustment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dns_record_adjustments(
    store: Rc<dyn DnsRecordAdjustmentStore>,
    query: &DnsRecordAdjustmentQuery,
) -> Vec<DnsRecordAdjustment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DnsRecordAdjustmentStore> = new_memory_dns_record_adjustment_store();
        let item = new_dns_record_adjustment("code-588", "name-749");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
