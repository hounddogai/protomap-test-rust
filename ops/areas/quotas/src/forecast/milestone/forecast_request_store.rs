use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps forecast requests by code.
pub trait ForecastRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ForecastRequest>;
    fn save(&self, item: &ForecastRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ForecastRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryForecastRequestStore {
    pub items: RefCell<BTreeMap<String, ForecastRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl ForecastRequestStore for MemoryForecastRequestStore {
    fn find(&self, code: &str) -> Option<ForecastRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ForecastRequest) {
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

    fn list(&self) -> Vec<ForecastRequest> {
        let mut out: Vec<ForecastRequest> = Vec::new();
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

pub fn new_memory_forecast_request_store() -> Rc<dyn ForecastRequestStore> {
    Rc::new(MemoryForecastRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedForecastRequestStore {
    pub inner: Rc<dyn ForecastRequestStore>,
    pub cache: RefCell<cache::TtlCache<ForecastRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ForecastRequestStore for CachedForecastRequestStore {
    fn find(&self, code: &str) -> Option<ForecastRequest> {
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

    fn save(&self, item: &ForecastRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ForecastRequest> {
        self.inner.list()
    }
}

pub fn new_cached_forecast_request_store(
    inner: Rc<dyn ForecastRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ForecastRequestStore> {
    Rc::new(CachedForecastRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ForecastRequest>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects forecast requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRequestQuery {
    pub name: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ForecastRequestQuery {
    pub fn matches(&self, item: &ForecastRequest) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[ForecastRequest]) -> Vec<ForecastRequest> {
        let matched = collections::filter(items, |candidate: &ForecastRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_forecast_requests(
    store: Rc<dyn ForecastRequestStore>,
    query: &ForecastRequestQuery,
) -> Vec<ForecastRequest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ForecastRequestStore> = new_memory_forecast_request_store();
        let item = new_forecast_request("code-417", "name-659");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
