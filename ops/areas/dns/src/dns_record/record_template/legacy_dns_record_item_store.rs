use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy dns record items by code.
pub trait LegacyDnsRecordItemStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyDnsRecordItem>;
    fn save(&self, item: &LegacyDnsRecordItem);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyDnsRecordItem>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyDnsRecordItemStore {
    pub items: RefCell<BTreeMap<String, LegacyDnsRecordItem>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyDnsRecordItemStore for MemoryLegacyDnsRecordItemStore {
    fn find(&self, code: &str) -> Option<LegacyDnsRecordItem> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyDnsRecordItem) {
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

    fn list(&self) -> Vec<LegacyDnsRecordItem> {
        let mut out: Vec<LegacyDnsRecordItem> = Vec::new();
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

pub fn new_memory_legacy_dns_record_item_store() -> Rc<dyn LegacyDnsRecordItemStore> {
    Rc::new(MemoryLegacyDnsRecordItemStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyDnsRecordItemStore {
    pub inner: Rc<dyn LegacyDnsRecordItemStore>,
    pub cache: RefCell<cache::TtlCache<LegacyDnsRecordItem>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyDnsRecordItemStore for CachedLegacyDnsRecordItemStore {
    fn find(&self, code: &str) -> Option<LegacyDnsRecordItem> {
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

    fn save(&self, item: &LegacyDnsRecordItem) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyDnsRecordItem> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_dns_record_item_store(
    inner: Rc<dyn LegacyDnsRecordItemStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyDnsRecordItemStore> {
    Rc::new(CachedLegacyDnsRecordItemStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyDnsRecordItem>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy dns record items. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDnsRecordItemQuery {
    pub country: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyDnsRecordItemQuery {
    pub fn matches(&self, item: &LegacyDnsRecordItem) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyDnsRecordItem]) -> Vec<LegacyDnsRecordItem> {
        let matched = collections::filter(items, |candidate: &LegacyDnsRecordItem| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_dns_record_items(
    store: Rc<dyn LegacyDnsRecordItemStore>,
    query: &LegacyDnsRecordItemQuery,
) -> Vec<LegacyDnsRecordItem> {
    query.apply(&store.list())
}
