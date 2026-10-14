use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient redaction forecasts by code.
pub trait LenientRedactionForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientRedactionForecast>;
    fn save(&self, item: &LenientRedactionForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientRedactionForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientRedactionForecastStore {
    pub items: RefCell<BTreeMap<String, LenientRedactionForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientRedactionForecastStore for MemoryLenientRedactionForecastStore {
    fn find(&self, code: &str) -> Option<LenientRedactionForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientRedactionForecast) {
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

    fn list(&self) -> Vec<LenientRedactionForecast> {
        let mut out: Vec<LenientRedactionForecast> = Vec::new();
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

pub fn new_memory_lenient_redaction_forecast_store() -> Rc<dyn LenientRedactionForecastStore> {
    Rc::new(MemoryLenientRedactionForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientRedactionForecastStore {
    pub inner: Rc<dyn LenientRedactionForecastStore>,
    pub cache: RefCell<cache::TtlCache<LenientRedactionForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientRedactionForecastStore for CachedLenientRedactionForecastStore {
    fn find(&self, code: &str) -> Option<LenientRedactionForecast> {
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

    fn save(&self, item: &LenientRedactionForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientRedactionForecast> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_redaction_forecast_store(
    inner: Rc<dyn LenientRedactionForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientRedactionForecastStore> {
    Rc::new(CachedLenientRedactionForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientRedactionForecast>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient redaction forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionForecastQuery {
    pub unit: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientRedactionForecastQuery {
    pub fn matches(&self, item: &LenientRedactionForecast) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientRedactionForecast]) -> Vec<LenientRedactionForecast> {
        let matched = collections::filter(items, |candidate: &LenientRedactionForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_redaction_forecasts(
    store: Rc<dyn LenientRedactionForecastStore>,
    query: &LenientRedactionForecastQuery,
) -> Vec<LenientRedactionForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientRedactionForecastStore> = new_memory_lenient_redaction_forecast_store();
        let item = new_lenient_redaction_forecast("code-12", "label-678");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
