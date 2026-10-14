use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps redaction forecasts by code.
pub trait RedactionForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RedactionForecast>;
    fn save(&self, item: &RedactionForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RedactionForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRedactionForecastStore {
    pub items: RefCell<BTreeMap<String, RedactionForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl RedactionForecastStore for MemoryRedactionForecastStore {
    fn find(&self, code: &str) -> Option<RedactionForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RedactionForecast) {
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

    fn list(&self) -> Vec<RedactionForecast> {
        let mut out: Vec<RedactionForecast> = Vec::new();
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

pub fn new_memory_redaction_forecast_store() -> Rc<dyn RedactionForecastStore> {
    Rc::new(MemoryRedactionForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRedactionForecastStore {
    pub inner: Rc<dyn RedactionForecastStore>,
    pub cache: RefCell<cache::TtlCache<RedactionForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RedactionForecastStore for CachedRedactionForecastStore {
    fn find(&self, code: &str) -> Option<RedactionForecast> {
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

    fn save(&self, item: &RedactionForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RedactionForecast> {
        self.inner.list()
    }
}

pub fn new_cached_redaction_forecast_store(
    inner: Rc<dyn RedactionForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RedactionForecastStore> {
    Rc::new(CachedRedactionForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RedactionForecast>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects redaction forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionForecastQuery {
    pub category: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RedactionForecastQuery {
    pub fn matches(&self, item: &RedactionForecast) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[RedactionForecast]) -> Vec<RedactionForecast> {
        let matched = collections::filter(items, |candidate: &RedactionForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_redaction_forecasts(
    store: Rc<dyn RedactionForecastStore>,
    query: &RedactionForecastQuery,
) -> Vec<RedactionForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RedactionForecastStore> = new_memory_redaction_forecast_store();
        let item = new_redaction_forecast("code-671", "name-645");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
