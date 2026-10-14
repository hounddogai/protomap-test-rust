use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy query variances by code.
pub trait LegacyQueryVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyQueryVariance>;
    fn save(&self, item: &LegacyQueryVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyQueryVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyQueryVarianceStore {
    pub items: RefCell<BTreeMap<String, LegacyQueryVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyQueryVarianceStore for MemoryLegacyQueryVarianceStore {
    fn find(&self, code: &str) -> Option<LegacyQueryVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyQueryVariance) {
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

    fn list(&self) -> Vec<LegacyQueryVariance> {
        let mut out: Vec<LegacyQueryVariance> = Vec::new();
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

pub fn new_memory_legacy_query_variance_store() -> Rc<dyn LegacyQueryVarianceStore> {
    Rc::new(MemoryLegacyQueryVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyQueryVarianceStore {
    pub inner: Rc<dyn LegacyQueryVarianceStore>,
    pub cache: RefCell<cache::TtlCache<LegacyQueryVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyQueryVarianceStore for CachedLegacyQueryVarianceStore {
    fn find(&self, code: &str) -> Option<LegacyQueryVariance> {
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

    fn save(&self, item: &LegacyQueryVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyQueryVariance> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_query_variance_store(
    inner: Rc<dyn LegacyQueryVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyQueryVarianceStore> {
    Rc::new(CachedLegacyQueryVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyQueryVariance>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy query variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQueryVarianceQuery {
    pub country: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyQueryVarianceQuery {
    pub fn matches(&self, item: &LegacyQueryVariance) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyQueryVariance]) -> Vec<LegacyQueryVariance> {
        let matched = collections::filter(items, |candidate: &LegacyQueryVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_query_variances(
    store: Rc<dyn LegacyQueryVarianceStore>,
    query: &LegacyQueryVarianceQuery,
) -> Vec<LegacyQueryVariance> {
    query.apply(&store.list())
}
