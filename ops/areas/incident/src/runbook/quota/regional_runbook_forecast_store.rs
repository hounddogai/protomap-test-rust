use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional runbook forecasts by code.
pub trait RegionalRunbookForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalRunbookForecast>;
    fn save(&self, item: &RegionalRunbookForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalRunbookForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalRunbookForecastStore {
    pub items: RefCell<BTreeMap<String, RegionalRunbookForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalRunbookForecastStore for MemoryRegionalRunbookForecastStore {
    fn find(&self, code: &str) -> Option<RegionalRunbookForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalRunbookForecast) {
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

    fn list(&self) -> Vec<RegionalRunbookForecast> {
        let mut out: Vec<RegionalRunbookForecast> = Vec::new();
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

pub fn new_memory_regional_runbook_forecast_store() -> Rc<dyn RegionalRunbookForecastStore> {
    Rc::new(MemoryRegionalRunbookForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalRunbookForecastStore {
    pub inner: Rc<dyn RegionalRunbookForecastStore>,
    pub cache: RefCell<cache::TtlCache<RegionalRunbookForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalRunbookForecastStore for CachedRegionalRunbookForecastStore {
    fn find(&self, code: &str) -> Option<RegionalRunbookForecast> {
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

    fn save(&self, item: &RegionalRunbookForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalRunbookForecast> {
        self.inner.list()
    }
}

pub fn new_cached_regional_runbook_forecast_store(
    inner: Rc<dyn RegionalRunbookForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalRunbookForecastStore> {
    Rc::new(CachedRegionalRunbookForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalRunbookForecast>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional runbook forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookForecastQuery {
    pub label: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalRunbookForecastQuery {
    pub fn matches(&self, item: &RegionalRunbookForecast) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalRunbookForecast]) -> Vec<RegionalRunbookForecast> {
        let matched = collections::filter(items, |candidate: &RegionalRunbookForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_runbook_forecasts(
    store: Rc<dyn RegionalRunbookForecastStore>,
    query: &RegionalRunbookForecastQuery,
) -> Vec<RegionalRunbookForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalRunbookForecastStore> = new_memory_regional_runbook_forecast_store();
        let item = new_regional_runbook_forecast("code-471", "label-913");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
