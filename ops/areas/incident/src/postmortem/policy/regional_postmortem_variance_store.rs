use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional postmortem variances by code.
pub trait RegionalPostmortemVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalPostmortemVariance>;
    fn save(&self, item: &RegionalPostmortemVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalPostmortemVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalPostmortemVarianceStore {
    pub items: RefCell<BTreeMap<String, RegionalPostmortemVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalPostmortemVarianceStore for MemoryRegionalPostmortemVarianceStore {
    fn find(&self, code: &str) -> Option<RegionalPostmortemVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalPostmortemVariance) {
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

    fn list(&self) -> Vec<RegionalPostmortemVariance> {
        let mut out: Vec<RegionalPostmortemVariance> = Vec::new();
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

pub fn new_memory_regional_postmortem_variance_store() -> Rc<dyn RegionalPostmortemVarianceStore> {
    Rc::new(MemoryRegionalPostmortemVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalPostmortemVarianceStore {
    pub inner: Rc<dyn RegionalPostmortemVarianceStore>,
    pub cache: RefCell<cache::TtlCache<RegionalPostmortemVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalPostmortemVarianceStore for CachedRegionalPostmortemVarianceStore {
    fn find(&self, code: &str) -> Option<RegionalPostmortemVariance> {
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

    fn save(&self, item: &RegionalPostmortemVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalPostmortemVariance> {
        self.inner.list()
    }
}

pub fn new_cached_regional_postmortem_variance_store(
    inner: Rc<dyn RegionalPostmortemVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalPostmortemVarianceStore> {
    Rc::new(CachedRegionalPostmortemVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalPostmortemVariance>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional postmortem variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPostmortemVarianceQuery {
    pub namespace: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalPostmortemVarianceQuery {
    pub fn matches(&self, item: &RegionalPostmortemVariance) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.expires_at < self.min_expires_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalPostmortemVariance]) -> Vec<RegionalPostmortemVariance> {
        let matched = collections::filter(items, |candidate: &RegionalPostmortemVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_postmortem_variances(
    store: Rc<dyn RegionalPostmortemVarianceStore>,
    query: &RegionalPostmortemVarianceQuery,
) -> Vec<RegionalPostmortemVariance> {
    query.apply(&store.list())
}
