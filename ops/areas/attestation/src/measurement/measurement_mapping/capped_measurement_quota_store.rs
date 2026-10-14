use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped measurement quotas by code.
pub trait CappedMeasurementQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedMeasurementQuota>;
    fn save(&self, item: &CappedMeasurementQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedMeasurementQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedMeasurementQuotaStore {
    pub items: RefCell<BTreeMap<String, CappedMeasurementQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedMeasurementQuotaStore for MemoryCappedMeasurementQuotaStore {
    fn find(&self, code: &str) -> Option<CappedMeasurementQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedMeasurementQuota) {
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

    fn list(&self) -> Vec<CappedMeasurementQuota> {
        let mut out: Vec<CappedMeasurementQuota> = Vec::new();
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

pub fn new_memory_capped_measurement_quota_store() -> Rc<dyn CappedMeasurementQuotaStore> {
    Rc::new(MemoryCappedMeasurementQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedMeasurementQuotaStore {
    pub inner: Rc<dyn CappedMeasurementQuotaStore>,
    pub cache: RefCell<cache::TtlCache<CappedMeasurementQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedMeasurementQuotaStore for CachedCappedMeasurementQuotaStore {
    fn find(&self, code: &str) -> Option<CappedMeasurementQuota> {
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

    fn save(&self, item: &CappedMeasurementQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedMeasurementQuota> {
        self.inner.list()
    }
}

pub fn new_cached_capped_measurement_quota_store(
    inner: Rc<dyn CappedMeasurementQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedMeasurementQuotaStore> {
    Rc::new(CachedCappedMeasurementQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedMeasurementQuota>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped measurement quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedMeasurementQuotaQuery {
    pub country: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedMeasurementQuotaQuery {
    pub fn matches(&self, item: &CappedMeasurementQuota) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedMeasurementQuota]) -> Vec<CappedMeasurementQuota> {
        let matched = collections::filter(items, |candidate: &CappedMeasurementQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_measurement_quotas(
    store: Rc<dyn CappedMeasurementQuotaStore>,
    query: &CappedMeasurementQuotaQuery,
) -> Vec<CappedMeasurementQuota> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedMeasurementQuotaStore> = new_memory_capped_measurement_quota_store();
        let item = new_capped_measurement_quota("code-128", "name-285");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
