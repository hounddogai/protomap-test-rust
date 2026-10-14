use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional query variances by code.
pub trait RegionalQueryVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalQueryVariance>;
    fn save(&self, item: &RegionalQueryVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalQueryVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalQueryVarianceStore {
    pub items: RefCell<BTreeMap<String, RegionalQueryVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalQueryVarianceStore for MemoryRegionalQueryVarianceStore {
    fn find(&self, code: &str) -> Option<RegionalQueryVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalQueryVariance) {
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

    fn list(&self) -> Vec<RegionalQueryVariance> {
        let mut out: Vec<RegionalQueryVariance> = Vec::new();
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

pub fn new_memory_regional_query_variance_store() -> Rc<dyn RegionalQueryVarianceStore> {
    Rc::new(MemoryRegionalQueryVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalQueryVarianceStore {
    pub inner: Rc<dyn RegionalQueryVarianceStore>,
    pub cache: RefCell<cache::TtlCache<RegionalQueryVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalQueryVarianceStore for CachedRegionalQueryVarianceStore {
    fn find(&self, code: &str) -> Option<RegionalQueryVariance> {
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

    fn save(&self, item: &RegionalQueryVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalQueryVariance> {
        self.inner.list()
    }
}

pub fn new_cached_regional_query_variance_store(
    inner: Rc<dyn RegionalQueryVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalQueryVarianceStore> {
    Rc::new(CachedRegionalQueryVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalQueryVariance>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional query variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryVarianceQuery {
    pub region: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalQueryVarianceQuery {
    pub fn matches(&self, item: &RegionalQueryVariance) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalQueryVariance]) -> Vec<RegionalQueryVariance> {
        let matched = collections::filter(items, |candidate: &RegionalQueryVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_query_variances(
    store: Rc<dyn RegionalQueryVarianceStore>,
    query: &RegionalQueryVarianceQuery,
) -> Vec<RegionalQueryVariance> {
    query.apply(&store.list())
}
