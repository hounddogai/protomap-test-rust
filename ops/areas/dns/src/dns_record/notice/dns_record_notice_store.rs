use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dns record notices by code.
pub trait DnsRecordNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DnsRecordNotice>;
    fn save(&self, item: &DnsRecordNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DnsRecordNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDnsRecordNoticeStore {
    pub items: RefCell<BTreeMap<String, DnsRecordNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl DnsRecordNoticeStore for MemoryDnsRecordNoticeStore {
    fn find(&self, code: &str) -> Option<DnsRecordNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DnsRecordNotice) {
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

    fn list(&self) -> Vec<DnsRecordNotice> {
        let mut out: Vec<DnsRecordNotice> = Vec::new();
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

pub fn new_memory_dns_record_notice_store() -> Rc<dyn DnsRecordNoticeStore> {
    Rc::new(MemoryDnsRecordNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDnsRecordNoticeStore {
    pub inner: Rc<dyn DnsRecordNoticeStore>,
    pub cache: RefCell<cache::TtlCache<DnsRecordNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DnsRecordNoticeStore for CachedDnsRecordNoticeStore {
    fn find(&self, code: &str) -> Option<DnsRecordNotice> {
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

    fn save(&self, item: &DnsRecordNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DnsRecordNotice> {
        self.inner.list()
    }
}

pub fn new_cached_dns_record_notice_store(
    inner: Rc<dyn DnsRecordNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DnsRecordNoticeStore> {
    Rc::new(CachedDnsRecordNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DnsRecordNotice>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dns record notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordNoticeQuery {
    pub name: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DnsRecordNoticeQuery {
    pub fn matches(&self, item: &DnsRecordNotice) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DnsRecordNotice]) -> Vec<DnsRecordNotice> {
        let matched = collections::filter(items, |candidate: &DnsRecordNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dns_record_notices(
    store: Rc<dyn DnsRecordNoticeStore>,
    query: &DnsRecordNoticeQuery,
) -> Vec<DnsRecordNotice> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DnsRecordNoticeStore> = new_memory_dns_record_notice_store();
        let item = new_dns_record_notice("code-880", "name-100");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
