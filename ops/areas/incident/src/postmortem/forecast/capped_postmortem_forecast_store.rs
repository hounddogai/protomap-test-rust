use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped postmortem forecasts by code.
pub trait CappedPostmortemForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedPostmortemForecast>;
    fn save(&self, item: &CappedPostmortemForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedPostmortemForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedPostmortemForecastStore {
    pub items: RefCell<BTreeMap<String, CappedPostmortemForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedPostmortemForecastStore for MemoryCappedPostmortemForecastStore {
    fn find(&self, code: &str) -> Option<CappedPostmortemForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedPostmortemForecast) {
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

    fn list(&self) -> Vec<CappedPostmortemForecast> {
        let mut out: Vec<CappedPostmortemForecast> = Vec::new();
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

pub fn new_memory_capped_postmortem_forecast_store() -> Rc<dyn CappedPostmortemForecastStore> {
    Rc::new(MemoryCappedPostmortemForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedPostmortemForecastStore {
    pub inner: Rc<dyn CappedPostmortemForecastStore>,
    pub cache: RefCell<cache::TtlCache<CappedPostmortemForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedPostmortemForecastStore for CachedCappedPostmortemForecastStore {
    fn find(&self, code: &str) -> Option<CappedPostmortemForecast> {
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

    fn save(&self, item: &CappedPostmortemForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedPostmortemForecast> {
        self.inner.list()
    }
}

pub fn new_cached_capped_postmortem_forecast_store(
    inner: Rc<dyn CappedPostmortemForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedPostmortemForecastStore> {
    Rc::new(CachedCappedPostmortemForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedPostmortemForecast>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped postmortem forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPostmortemForecastQuery {
    pub namespace: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedPostmortemForecastQuery {
    pub fn matches(&self, item: &CappedPostmortemForecast) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedPostmortemForecast]) -> Vec<CappedPostmortemForecast> {
        let matched = collections::filter(items, |candidate: &CappedPostmortemForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_postmortem_forecasts(
    store: Rc<dyn CappedPostmortemForecastStore>,
    query: &CappedPostmortemForecastQuery,
) -> Vec<CappedPostmortemForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedPostmortemForecastStore> = new_memory_capped_postmortem_forecast_store();
        let item = new_capped_postmortem_forecast("code-448", "label-727");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
