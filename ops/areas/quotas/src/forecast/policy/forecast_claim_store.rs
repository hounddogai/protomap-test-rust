use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps forecast claims by code.
pub trait ForecastClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ForecastClaim>;
    fn save(&self, item: &ForecastClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ForecastClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryForecastClaimStore {
    pub items: RefCell<BTreeMap<String, ForecastClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl ForecastClaimStore for MemoryForecastClaimStore {
    fn find(&self, code: &str) -> Option<ForecastClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ForecastClaim) {
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

    fn list(&self) -> Vec<ForecastClaim> {
        let mut out: Vec<ForecastClaim> = Vec::new();
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

pub fn new_memory_forecast_claim_store() -> Rc<dyn ForecastClaimStore> {
    Rc::new(MemoryForecastClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedForecastClaimStore {
    pub inner: Rc<dyn ForecastClaimStore>,
    pub cache: RefCell<cache::TtlCache<ForecastClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ForecastClaimStore for CachedForecastClaimStore {
    fn find(&self, code: &str) -> Option<ForecastClaim> {
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

    fn save(&self, item: &ForecastClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ForecastClaim> {
        self.inner.list()
    }
}

pub fn new_cached_forecast_claim_store(
    inner: Rc<dyn ForecastClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ForecastClaimStore> {
    Rc::new(CachedForecastClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ForecastClaim>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects forecast claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastClaimQuery {
    pub cluster: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ForecastClaimQuery {
    pub fn matches(&self, item: &ForecastClaim) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ForecastClaim]) -> Vec<ForecastClaim> {
        let matched = collections::filter(items, |candidate: &ForecastClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_forecast_claims(store: Rc<dyn ForecastClaimStore>, query: &ForecastClaimQuery) -> Vec<ForecastClaim> {
    query.apply(&store.list())
}
