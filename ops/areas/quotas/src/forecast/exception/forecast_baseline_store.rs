use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps forecast baselines by code.
pub trait ForecastBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ForecastBaseline>;
    fn save(&self, item: &ForecastBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ForecastBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryForecastBaselineStore {
    pub items: RefCell<BTreeMap<String, ForecastBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl ForecastBaselineStore for MemoryForecastBaselineStore {
    fn find(&self, code: &str) -> Option<ForecastBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ForecastBaseline) {
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

    fn list(&self) -> Vec<ForecastBaseline> {
        let mut out: Vec<ForecastBaseline> = Vec::new();
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

pub fn new_memory_forecast_baseline_store() -> Rc<dyn ForecastBaselineStore> {
    Rc::new(MemoryForecastBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedForecastBaselineStore {
    pub inner: Rc<dyn ForecastBaselineStore>,
    pub cache: RefCell<cache::TtlCache<ForecastBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ForecastBaselineStore for CachedForecastBaselineStore {
    fn find(&self, code: &str) -> Option<ForecastBaseline> {
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

    fn save(&self, item: &ForecastBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ForecastBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_forecast_baseline_store(
    inner: Rc<dyn ForecastBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ForecastBaselineStore> {
    Rc::new(CachedForecastBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ForecastBaseline>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects forecast baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBaselineQuery {
    pub source: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ForecastBaselineQuery {
    pub fn matches(&self, item: &ForecastBaseline) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
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

    pub fn apply(&self, items: &[ForecastBaseline]) -> Vec<ForecastBaseline> {
        let matched = collections::filter(items, |candidate: &ForecastBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_forecast_baselines(
    store: Rc<dyn ForecastBaselineStore>,
    query: &ForecastBaselineQuery,
) -> Vec<ForecastBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ForecastBaselineStore> = new_memory_forecast_baseline_store();
        let item = new_forecast_baseline("code-919", "name-542");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
