use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal limit metrics by code.
pub trait SeasonalLimitMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalLimitMetric>;
    fn save(&self, item: &SeasonalLimitMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalLimitMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalLimitMetricStore {
    pub items: RefCell<BTreeMap<String, SeasonalLimitMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalLimitMetricStore for MemorySeasonalLimitMetricStore {
    fn find(&self, code: &str) -> Option<SeasonalLimitMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalLimitMetric) {
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

    fn list(&self) -> Vec<SeasonalLimitMetric> {
        let mut out: Vec<SeasonalLimitMetric> = Vec::new();
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

pub fn new_memory_seasonal_limit_metric_store() -> Rc<dyn SeasonalLimitMetricStore> {
    Rc::new(MemorySeasonalLimitMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalLimitMetricStore {
    pub inner: Rc<dyn SeasonalLimitMetricStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalLimitMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalLimitMetricStore for CachedSeasonalLimitMetricStore {
    fn find(&self, code: &str) -> Option<SeasonalLimitMetric> {
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

    fn save(&self, item: &SeasonalLimitMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalLimitMetric> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_limit_metric_store(
    inner: Rc<dyn SeasonalLimitMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalLimitMetricStore> {
    Rc::new(CachedSeasonalLimitMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalLimitMetric>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal limit metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalLimitMetricQuery {
    pub name: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalLimitMetricQuery {
    pub fn matches(&self, item: &SeasonalLimitMetric) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[SeasonalLimitMetric]) -> Vec<SeasonalLimitMetric> {
        let matched = collections::filter(items, |candidate: &SeasonalLimitMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_limit_metrics(
    store: Rc<dyn SeasonalLimitMetricStore>,
    query: &SeasonalLimitMetricQuery,
) -> Vec<SeasonalLimitMetric> {
    query.apply(&store.list())
}
