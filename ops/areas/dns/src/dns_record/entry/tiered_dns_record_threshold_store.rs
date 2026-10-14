use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered dns record thresholds by code.
pub trait TieredDnsRecordThresholdStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredDnsRecordThreshold>;
    fn save(&self, item: &TieredDnsRecordThreshold);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredDnsRecordThreshold>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredDnsRecordThresholdStore {
    pub items: RefCell<BTreeMap<String, TieredDnsRecordThreshold>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredDnsRecordThresholdStore for MemoryTieredDnsRecordThresholdStore {
    fn find(&self, code: &str) -> Option<TieredDnsRecordThreshold> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredDnsRecordThreshold) {
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

    fn list(&self) -> Vec<TieredDnsRecordThreshold> {
        let mut out: Vec<TieredDnsRecordThreshold> = Vec::new();
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

pub fn new_memory_tiered_dns_record_threshold_store() -> Rc<dyn TieredDnsRecordThresholdStore> {
    Rc::new(MemoryTieredDnsRecordThresholdStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredDnsRecordThresholdStore {
    pub inner: Rc<dyn TieredDnsRecordThresholdStore>,
    pub cache: RefCell<cache::TtlCache<TieredDnsRecordThreshold>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredDnsRecordThresholdStore for CachedTieredDnsRecordThresholdStore {
    fn find(&self, code: &str) -> Option<TieredDnsRecordThreshold> {
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

    fn save(&self, item: &TieredDnsRecordThreshold) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredDnsRecordThreshold> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_dns_record_threshold_store(
    inner: Rc<dyn TieredDnsRecordThresholdStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredDnsRecordThresholdStore> {
    Rc::new(CachedTieredDnsRecordThresholdStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredDnsRecordThreshold>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered dns record thresholds. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordThresholdQuery {
    pub name: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredDnsRecordThresholdQuery {
    pub fn matches(&self, item: &TieredDnsRecordThreshold) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredDnsRecordThreshold]) -> Vec<TieredDnsRecordThreshold> {
        let matched = collections::filter(items, |candidate: &TieredDnsRecordThreshold| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_dns_record_thresholds(
    store: Rc<dyn TieredDnsRecordThresholdStore>,
    query: &TieredDnsRecordThresholdQuery,
) -> Vec<TieredDnsRecordThreshold> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredDnsRecordThresholdStore> = new_memory_tiered_dns_record_threshold_store();
        let item = new_tiered_dns_record_threshold("code-872", "name-804");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
