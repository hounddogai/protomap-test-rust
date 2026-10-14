use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional quote metrics by code.
pub trait RegionalQuoteMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalQuoteMetric>;
    fn save(&self, item: &RegionalQuoteMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalQuoteMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalQuoteMetricStore {
    pub items: RefCell<BTreeMap<String, RegionalQuoteMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalQuoteMetricStore for MemoryRegionalQuoteMetricStore {
    fn find(&self, code: &str) -> Option<RegionalQuoteMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalQuoteMetric) {
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

    fn list(&self) -> Vec<RegionalQuoteMetric> {
        let mut out: Vec<RegionalQuoteMetric> = Vec::new();
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

pub fn new_memory_regional_quote_metric_store() -> Rc<dyn RegionalQuoteMetricStore> {
    Rc::new(MemoryRegionalQuoteMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalQuoteMetricStore {
    pub inner: Rc<dyn RegionalQuoteMetricStore>,
    pub cache: RefCell<cache::TtlCache<RegionalQuoteMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalQuoteMetricStore for CachedRegionalQuoteMetricStore {
    fn find(&self, code: &str) -> Option<RegionalQuoteMetric> {
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

    fn save(&self, item: &RegionalQuoteMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalQuoteMetric> {
        self.inner.list()
    }
}

pub fn new_cached_regional_quote_metric_store(
    inner: Rc<dyn RegionalQuoteMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalQuoteMetricStore> {
    Rc::new(CachedRegionalQuoteMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalQuoteMetric>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional quote metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQuoteMetricQuery {
    pub title: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalQuoteMetricQuery {
    pub fn matches(&self, item: &RegionalQuoteMetric) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalQuoteMetric]) -> Vec<RegionalQuoteMetric> {
        let matched = collections::filter(items, |candidate: &RegionalQuoteMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_quote_metrics(
    store: Rc<dyn RegionalQuoteMetricStore>,
    query: &RegionalQuoteMetricQuery,
) -> Vec<RegionalQuoteMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalQuoteMetricStore> = new_memory_regional_quote_metric_store();
        let item = new_regional_quote_metric("code-257", "title-923");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
