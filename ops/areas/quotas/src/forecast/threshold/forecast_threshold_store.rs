use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps forecast thresholds by code.
pub trait ForecastThresholdStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ForecastThreshold>;
    fn save(&self, item: &ForecastThreshold);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ForecastThreshold>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryForecastThresholdStore {
    pub items: RefCell<BTreeMap<String, ForecastThreshold>>,
    pub order: RefCell<Vec<String>>,
}

impl ForecastThresholdStore for MemoryForecastThresholdStore {
    fn find(&self, code: &str) -> Option<ForecastThreshold> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ForecastThreshold) {
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

    fn list(&self) -> Vec<ForecastThreshold> {
        let mut out: Vec<ForecastThreshold> = Vec::new();
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

pub fn new_memory_forecast_threshold_store() -> Rc<dyn ForecastThresholdStore> {
    Rc::new(MemoryForecastThresholdStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedForecastThresholdStore {
    pub inner: Rc<dyn ForecastThresholdStore>,
    pub cache: RefCell<cache::TtlCache<ForecastThreshold>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ForecastThresholdStore for CachedForecastThresholdStore {
    fn find(&self, code: &str) -> Option<ForecastThreshold> {
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

    fn save(&self, item: &ForecastThreshold) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ForecastThreshold> {
        self.inner.list()
    }
}

pub fn new_cached_forecast_threshold_store(
    inner: Rc<dyn ForecastThresholdStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ForecastThresholdStore> {
    Rc::new(CachedForecastThresholdStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ForecastThreshold>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects forecast thresholds. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastThresholdQuery {
    pub name: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ForecastThresholdQuery {
    pub fn matches(&self, item: &ForecastThreshold) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ForecastThreshold]) -> Vec<ForecastThreshold> {
        let matched = collections::filter(items, |candidate: &ForecastThreshold| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_forecast_thresholds(
    store: Rc<dyn ForecastThresholdStore>,
    query: &ForecastThresholdQuery,
) -> Vec<ForecastThreshold> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ForecastThresholdStore> = new_memory_forecast_threshold_store();
        let item = new_forecast_threshold("code-309", "name-481");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
