use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps quote forecasts by code.
pub trait QuoteForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QuoteForecast>;
    fn save(&self, item: &QuoteForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QuoteForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQuoteForecastStore {
    pub items: RefCell<BTreeMap<String, QuoteForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl QuoteForecastStore for MemoryQuoteForecastStore {
    fn find(&self, code: &str) -> Option<QuoteForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QuoteForecast) {
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

    fn list(&self) -> Vec<QuoteForecast> {
        let mut out: Vec<QuoteForecast> = Vec::new();
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

pub fn new_memory_quote_forecast_store() -> Rc<dyn QuoteForecastStore> {
    Rc::new(MemoryQuoteForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQuoteForecastStore {
    pub inner: Rc<dyn QuoteForecastStore>,
    pub cache: RefCell<cache::TtlCache<QuoteForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QuoteForecastStore for CachedQuoteForecastStore {
    fn find(&self, code: &str) -> Option<QuoteForecast> {
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

    fn save(&self, item: &QuoteForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QuoteForecast> {
        self.inner.list()
    }
}

pub fn new_cached_quote_forecast_store(
    inner: Rc<dyn QuoteForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QuoteForecastStore> {
    Rc::new(CachedQuoteForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QuoteForecast>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects quote forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteForecastQuery {
    pub source: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QuoteForecastQuery {
    pub fn matches(&self, item: &QuoteForecast) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[QuoteForecast]) -> Vec<QuoteForecast> {
        let matched = collections::filter(items, |candidate: &QuoteForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_quote_forecasts(store: Rc<dyn QuoteForecastStore>, query: &QuoteForecastQuery) -> Vec<QuoteForecast> {
    query.apply(&store.list())
}
