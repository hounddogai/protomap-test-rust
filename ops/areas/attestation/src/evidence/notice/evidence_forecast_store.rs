use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps evidence forecasts by code.
pub trait EvidenceForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EvidenceForecast>;
    fn save(&self, item: &EvidenceForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EvidenceForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEvidenceForecastStore {
    pub items: RefCell<BTreeMap<String, EvidenceForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl EvidenceForecastStore for MemoryEvidenceForecastStore {
    fn find(&self, code: &str) -> Option<EvidenceForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EvidenceForecast) {
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

    fn list(&self) -> Vec<EvidenceForecast> {
        let mut out: Vec<EvidenceForecast> = Vec::new();
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

pub fn new_memory_evidence_forecast_store() -> Rc<dyn EvidenceForecastStore> {
    Rc::new(MemoryEvidenceForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEvidenceForecastStore {
    pub inner: Rc<dyn EvidenceForecastStore>,
    pub cache: RefCell<cache::TtlCache<EvidenceForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EvidenceForecastStore for CachedEvidenceForecastStore {
    fn find(&self, code: &str) -> Option<EvidenceForecast> {
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

    fn save(&self, item: &EvidenceForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EvidenceForecast> {
        self.inner.list()
    }
}

pub fn new_cached_evidence_forecast_store(
    inner: Rc<dyn EvidenceForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EvidenceForecastStore> {
    Rc::new(CachedEvidenceForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EvidenceForecast>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects evidence forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceForecastQuery {
    pub namespace: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EvidenceForecastQuery {
    pub fn matches(&self, item: &EvidenceForecast) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[EvidenceForecast]) -> Vec<EvidenceForecast> {
        let matched = collections::filter(items, |candidate: &EvidenceForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_evidence_forecasts(
    store: Rc<dyn EvidenceForecastStore>,
    query: &EvidenceForecastQuery,
) -> Vec<EvidenceForecast> {
    query.apply(&store.list())
}
