use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback data export quotas by code.
pub trait FallbackDataExportQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackDataExportQuota>;
    fn save(&self, item: &FallbackDataExportQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackDataExportQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackDataExportQuotaStore {
    pub items: RefCell<BTreeMap<String, FallbackDataExportQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackDataExportQuotaStore for MemoryFallbackDataExportQuotaStore {
    fn find(&self, code: &str) -> Option<FallbackDataExportQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackDataExportQuota) {
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

    fn list(&self) -> Vec<FallbackDataExportQuota> {
        let mut out: Vec<FallbackDataExportQuota> = Vec::new();
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

pub fn new_memory_fallback_data_export_quota_store() -> Rc<dyn FallbackDataExportQuotaStore> {
    Rc::new(MemoryFallbackDataExportQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackDataExportQuotaStore {
    pub inner: Rc<dyn FallbackDataExportQuotaStore>,
    pub cache: RefCell<cache::TtlCache<FallbackDataExportQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackDataExportQuotaStore for CachedFallbackDataExportQuotaStore {
    fn find(&self, code: &str) -> Option<FallbackDataExportQuota> {
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

    fn save(&self, item: &FallbackDataExportQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackDataExportQuota> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_data_export_quota_store(
    inner: Rc<dyn FallbackDataExportQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackDataExportQuotaStore> {
    Rc::new(CachedFallbackDataExportQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackDataExportQuota>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback data export quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportQuotaQuery {
    pub region: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackDataExportQuotaQuery {
    pub fn matches(&self, item: &FallbackDataExportQuota) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[FallbackDataExportQuota]) -> Vec<FallbackDataExportQuota> {
        let matched = collections::filter(items, |candidate: &FallbackDataExportQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_data_export_quotas(
    store: Rc<dyn FallbackDataExportQuotaStore>,
    query: &FallbackDataExportQuotaQuery,
) -> Vec<FallbackDataExportQuota> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackDataExportQuotaStore> = new_memory_fallback_data_export_quota_store();
        let item = new_fallback_data_export_quota("code-993", "label-513");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
