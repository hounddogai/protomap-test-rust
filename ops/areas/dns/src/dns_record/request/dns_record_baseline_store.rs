use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dns record baselines by code.
pub trait DnsRecordBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DnsRecordBaseline>;
    fn save(&self, item: &DnsRecordBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DnsRecordBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDnsRecordBaselineStore {
    pub items: RefCell<BTreeMap<String, DnsRecordBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl DnsRecordBaselineStore for MemoryDnsRecordBaselineStore {
    fn find(&self, code: &str) -> Option<DnsRecordBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DnsRecordBaseline) {
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

    fn list(&self) -> Vec<DnsRecordBaseline> {
        let mut out: Vec<DnsRecordBaseline> = Vec::new();
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

pub fn new_memory_dns_record_baseline_store() -> Rc<dyn DnsRecordBaselineStore> {
    Rc::new(MemoryDnsRecordBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDnsRecordBaselineStore {
    pub inner: Rc<dyn DnsRecordBaselineStore>,
    pub cache: RefCell<cache::TtlCache<DnsRecordBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DnsRecordBaselineStore for CachedDnsRecordBaselineStore {
    fn find(&self, code: &str) -> Option<DnsRecordBaseline> {
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

    fn save(&self, item: &DnsRecordBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DnsRecordBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_dns_record_baseline_store(
    inner: Rc<dyn DnsRecordBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DnsRecordBaselineStore> {
    Rc::new(CachedDnsRecordBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DnsRecordBaseline>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dns record baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBaselineQuery {
    pub title: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DnsRecordBaselineQuery {
    pub fn matches(&self, item: &DnsRecordBaseline) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DnsRecordBaseline]) -> Vec<DnsRecordBaseline> {
        let matched = collections::filter(items, |candidate: &DnsRecordBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dns_record_baselines(
    store: Rc<dyn DnsRecordBaselineStore>,
    query: &DnsRecordBaselineQuery,
) -> Vec<DnsRecordBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DnsRecordBaselineStore> = new_memory_dns_record_baseline_store();
        let item = new_dns_record_baseline("code-979", "title-269");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
