use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual forecast summaries by code.
pub trait ManualForecastSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualForecastSummary>;
    fn save(&self, item: &ManualForecastSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualForecastSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualForecastSummaryStore {
    pub items: RefCell<BTreeMap<String, ManualForecastSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualForecastSummaryStore for MemoryManualForecastSummaryStore {
    fn find(&self, code: &str) -> Option<ManualForecastSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualForecastSummary) {
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

    fn list(&self) -> Vec<ManualForecastSummary> {
        let mut out: Vec<ManualForecastSummary> = Vec::new();
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

pub fn new_memory_manual_forecast_summary_store() -> Rc<dyn ManualForecastSummaryStore> {
    Rc::new(MemoryManualForecastSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualForecastSummaryStore {
    pub inner: Rc<dyn ManualForecastSummaryStore>,
    pub cache: RefCell<cache::TtlCache<ManualForecastSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualForecastSummaryStore for CachedManualForecastSummaryStore {
    fn find(&self, code: &str) -> Option<ManualForecastSummary> {
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

    fn save(&self, item: &ManualForecastSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualForecastSummary> {
        self.inner.list()
    }
}

pub fn new_cached_manual_forecast_summary_store(
    inner: Rc<dyn ManualForecastSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualForecastSummaryStore> {
    Rc::new(CachedManualForecastSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualForecastSummary>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual forecast summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastSummaryQuery {
    pub cluster: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualForecastSummaryQuery {
    pub fn matches(&self, item: &ManualForecastSummary) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualForecastSummary]) -> Vec<ManualForecastSummary> {
        let matched = collections::filter(items, |candidate: &ManualForecastSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_forecast_summaries(
    store: Rc<dyn ManualForecastSummaryStore>,
    query: &ManualForecastSummaryQuery,
) -> Vec<ManualForecastSummary> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualForecastSummaryStore> = new_memory_manual_forecast_summary_store();
        let item = new_manual_forecast_summary("code-641", "name-358");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
