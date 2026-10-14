use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps suspension forecasts by code.
pub trait SuspensionForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SuspensionForecast>;
    fn save(&self, item: &SuspensionForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SuspensionForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySuspensionForecastStore {
    pub items: RefCell<BTreeMap<String, SuspensionForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl SuspensionForecastStore for MemorySuspensionForecastStore {
    fn find(&self, code: &str) -> Option<SuspensionForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SuspensionForecast) {
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

    fn list(&self) -> Vec<SuspensionForecast> {
        let mut out: Vec<SuspensionForecast> = Vec::new();
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

pub fn new_memory_suspension_forecast_store() -> Rc<dyn SuspensionForecastStore> {
    Rc::new(MemorySuspensionForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSuspensionForecastStore {
    pub inner: Rc<dyn SuspensionForecastStore>,
    pub cache: RefCell<cache::TtlCache<SuspensionForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SuspensionForecastStore for CachedSuspensionForecastStore {
    fn find(&self, code: &str) -> Option<SuspensionForecast> {
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

    fn save(&self, item: &SuspensionForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SuspensionForecast> {
        self.inner.list()
    }
}

pub fn new_cached_suspension_forecast_store(
    inner: Rc<dyn SuspensionForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SuspensionForecastStore> {
    Rc::new(CachedSuspensionForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SuspensionForecast>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects suspension forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionForecastQuery {
    pub namespace: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SuspensionForecastQuery {
    pub fn matches(&self, item: &SuspensionForecast) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.cost < self.min_cost {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SuspensionForecast]) -> Vec<SuspensionForecast> {
        let matched = collections::filter(items, |candidate: &SuspensionForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_suspension_forecasts(
    store: Rc<dyn SuspensionForecastStore>,
    query: &SuspensionForecastQuery,
) -> Vec<SuspensionForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SuspensionForecastStore> = new_memory_suspension_forecast_store();
        let item = new_suspension_forecast("code-434", "name-560");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
