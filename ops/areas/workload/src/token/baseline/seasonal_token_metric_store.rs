use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal token metrics by code.
pub trait SeasonalTokenMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalTokenMetric>;
    fn save(&self, item: &SeasonalTokenMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalTokenMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalTokenMetricStore {
    pub items: RefCell<BTreeMap<String, SeasonalTokenMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalTokenMetricStore for MemorySeasonalTokenMetricStore {
    fn find(&self, code: &str) -> Option<SeasonalTokenMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalTokenMetric) {
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

    fn list(&self) -> Vec<SeasonalTokenMetric> {
        let mut out: Vec<SeasonalTokenMetric> = Vec::new();
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

pub fn new_memory_seasonal_token_metric_store() -> Rc<dyn SeasonalTokenMetricStore> {
    Rc::new(MemorySeasonalTokenMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalTokenMetricStore {
    pub inner: Rc<dyn SeasonalTokenMetricStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalTokenMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalTokenMetricStore for CachedSeasonalTokenMetricStore {
    fn find(&self, code: &str) -> Option<SeasonalTokenMetric> {
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

    fn save(&self, item: &SeasonalTokenMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalTokenMetric> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_token_metric_store(
    inner: Rc<dyn SeasonalTokenMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalTokenMetricStore> {
    Rc::new(CachedSeasonalTokenMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalTokenMetric>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal token metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenMetricQuery {
    pub locale: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalTokenMetricQuery {
    pub fn matches(&self, item: &SeasonalTokenMetric) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalTokenMetric]) -> Vec<SeasonalTokenMetric> {
        let matched = collections::filter(items, |candidate: &SeasonalTokenMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_token_metrics(
    store: Rc<dyn SeasonalTokenMetricStore>,
    query: &SeasonalTokenMetricQuery,
) -> Vec<SeasonalTokenMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalTokenMetricStore> = new_memory_seasonal_token_metric_store();
        let item = new_seasonal_token_metric("code-737", "title-386");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
