use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual timeline forecasts by code.
pub trait ManualTimelineForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualTimelineForecast>;
    fn save(&self, item: &ManualTimelineForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualTimelineForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualTimelineForecastStore {
    pub items: RefCell<BTreeMap<String, ManualTimelineForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualTimelineForecastStore for MemoryManualTimelineForecastStore {
    fn find(&self, code: &str) -> Option<ManualTimelineForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualTimelineForecast) {
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

    fn list(&self) -> Vec<ManualTimelineForecast> {
        let mut out: Vec<ManualTimelineForecast> = Vec::new();
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

pub fn new_memory_manual_timeline_forecast_store() -> Rc<dyn ManualTimelineForecastStore> {
    Rc::new(MemoryManualTimelineForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualTimelineForecastStore {
    pub inner: Rc<dyn ManualTimelineForecastStore>,
    pub cache: RefCell<cache::TtlCache<ManualTimelineForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualTimelineForecastStore for CachedManualTimelineForecastStore {
    fn find(&self, code: &str) -> Option<ManualTimelineForecast> {
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

    fn save(&self, item: &ManualTimelineForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualTimelineForecast> {
        self.inner.list()
    }
}

pub fn new_cached_manual_timeline_forecast_store(
    inner: Rc<dyn ManualTimelineForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualTimelineForecastStore> {
    Rc::new(CachedManualTimelineForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualTimelineForecast>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual timeline forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTimelineForecastQuery {
    pub locale: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualTimelineForecastQuery {
    pub fn matches(&self, item: &ManualTimelineForecast) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualTimelineForecast]) -> Vec<ManualTimelineForecast> {
        let matched = collections::filter(items, |candidate: &ManualTimelineForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_timeline_forecasts(
    store: Rc<dyn ManualTimelineForecastStore>,
    query: &ManualTimelineForecastQuery,
) -> Vec<ManualTimelineForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualTimelineForecastStore> = new_memory_manual_timeline_forecast_store();
        let item = new_manual_timeline_forecast("code-488", "title-933");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
