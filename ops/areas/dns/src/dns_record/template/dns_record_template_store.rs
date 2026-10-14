use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dns record templates by code.
pub trait DnsRecordTemplateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DnsRecordTemplate>;
    fn save(&self, item: &DnsRecordTemplate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DnsRecordTemplate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDnsRecordTemplateStore {
    pub items: RefCell<BTreeMap<String, DnsRecordTemplate>>,
    pub order: RefCell<Vec<String>>,
}

impl DnsRecordTemplateStore for MemoryDnsRecordTemplateStore {
    fn find(&self, code: &str) -> Option<DnsRecordTemplate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DnsRecordTemplate) {
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

    fn list(&self) -> Vec<DnsRecordTemplate> {
        let mut out: Vec<DnsRecordTemplate> = Vec::new();
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

pub fn new_memory_dns_record_template_store() -> Rc<dyn DnsRecordTemplateStore> {
    Rc::new(MemoryDnsRecordTemplateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDnsRecordTemplateStore {
    pub inner: Rc<dyn DnsRecordTemplateStore>,
    pub cache: RefCell<cache::TtlCache<DnsRecordTemplate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DnsRecordTemplateStore for CachedDnsRecordTemplateStore {
    fn find(&self, code: &str) -> Option<DnsRecordTemplate> {
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

    fn save(&self, item: &DnsRecordTemplate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DnsRecordTemplate> {
        self.inner.list()
    }
}

pub fn new_cached_dns_record_template_store(
    inner: Rc<dyn DnsRecordTemplateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DnsRecordTemplateStore> {
    Rc::new(CachedDnsRecordTemplateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DnsRecordTemplate>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dns record templates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTemplateQuery {
    pub region: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DnsRecordTemplateQuery {
    pub fn matches(&self, item: &DnsRecordTemplate) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DnsRecordTemplate]) -> Vec<DnsRecordTemplate> {
        let matched = collections::filter(items, |candidate: &DnsRecordTemplate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dns_record_templates(
    store: Rc<dyn DnsRecordTemplateStore>,
    query: &DnsRecordTemplateQuery,
) -> Vec<DnsRecordTemplate> {
    query.apply(&store.list())
}
