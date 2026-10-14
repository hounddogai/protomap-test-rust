use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps action item forecasts by code.
pub trait ActionItemForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ActionItemForecast>;
    fn save(&self, item: &ActionItemForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ActionItemForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryActionItemForecastStore {
    pub items: RefCell<BTreeMap<String, ActionItemForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl ActionItemForecastStore for MemoryActionItemForecastStore {
    fn find(&self, code: &str) -> Option<ActionItemForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ActionItemForecast) {
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

    fn list(&self) -> Vec<ActionItemForecast> {
        let mut out: Vec<ActionItemForecast> = Vec::new();
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

pub fn new_memory_action_item_forecast_store() -> Rc<dyn ActionItemForecastStore> {
    Rc::new(MemoryActionItemForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedActionItemForecastStore {
    pub inner: Rc<dyn ActionItemForecastStore>,
    pub cache: RefCell<cache::TtlCache<ActionItemForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ActionItemForecastStore for CachedActionItemForecastStore {
    fn find(&self, code: &str) -> Option<ActionItemForecast> {
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

    fn save(&self, item: &ActionItemForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ActionItemForecast> {
        self.inner.list()
    }
}

pub fn new_cached_action_item_forecast_store(
    inner: Rc<dyn ActionItemForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ActionItemForecastStore> {
    Rc::new(CachedActionItemForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ActionItemForecast>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects action item forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemForecastQuery {
    pub label: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ActionItemForecastQuery {
    pub fn matches(&self, item: &ActionItemForecast) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[ActionItemForecast]) -> Vec<ActionItemForecast> {
        let matched = collections::filter(items, |candidate: &ActionItemForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_action_item_forecasts(
    store: Rc<dyn ActionItemForecastStore>,
    query: &ActionItemForecastQuery,
) -> Vec<ActionItemForecast> {
    query.apply(&store.list())
}
