use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps approval forecasts by code.
pub trait ApprovalForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ApprovalForecast>;
    fn save(&self, item: &ApprovalForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ApprovalForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryApprovalForecastStore {
    pub items: RefCell<BTreeMap<String, ApprovalForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl ApprovalForecastStore for MemoryApprovalForecastStore {
    fn find(&self, code: &str) -> Option<ApprovalForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ApprovalForecast) {
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

    fn list(&self) -> Vec<ApprovalForecast> {
        let mut out: Vec<ApprovalForecast> = Vec::new();
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

pub fn new_memory_approval_forecast_store() -> Rc<dyn ApprovalForecastStore> {
    Rc::new(MemoryApprovalForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedApprovalForecastStore {
    pub inner: Rc<dyn ApprovalForecastStore>,
    pub cache: RefCell<cache::TtlCache<ApprovalForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ApprovalForecastStore for CachedApprovalForecastStore {
    fn find(&self, code: &str) -> Option<ApprovalForecast> {
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

    fn save(&self, item: &ApprovalForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ApprovalForecast> {
        self.inner.list()
    }
}

pub fn new_cached_approval_forecast_store(
    inner: Rc<dyn ApprovalForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ApprovalForecastStore> {
    Rc::new(CachedApprovalForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ApprovalForecast>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects approval forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalForecastQuery {
    pub locale: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ApprovalForecastQuery {
    pub fn matches(&self, item: &ApprovalForecast) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ApprovalForecast]) -> Vec<ApprovalForecast> {
        let matched = collections::filter(items, |candidate: &ApprovalForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_approval_forecasts(
    store: Rc<dyn ApprovalForecastStore>,
    query: &ApprovalForecastQuery,
) -> Vec<ApprovalForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ApprovalForecastStore> = new_memory_approval_forecast_store();
        let item = new_approval_forecast("code-16", "title-541");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
