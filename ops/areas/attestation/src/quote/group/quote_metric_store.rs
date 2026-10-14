use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps quote metrics by code.
pub trait QuoteMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QuoteMetric>;
    fn save(&self, item: &QuoteMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QuoteMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQuoteMetricStore {
    pub items: RefCell<BTreeMap<String, QuoteMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl QuoteMetricStore for MemoryQuoteMetricStore {
    fn find(&self, code: &str) -> Option<QuoteMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QuoteMetric) {
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

    fn list(&self) -> Vec<QuoteMetric> {
        let mut out: Vec<QuoteMetric> = Vec::new();
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

pub fn new_memory_quote_metric_store() -> Rc<dyn QuoteMetricStore> {
    Rc::new(MemoryQuoteMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQuoteMetricStore {
    pub inner: Rc<dyn QuoteMetricStore>,
    pub cache: RefCell<cache::TtlCache<QuoteMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QuoteMetricStore for CachedQuoteMetricStore {
    fn find(&self, code: &str) -> Option<QuoteMetric> {
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

    fn save(&self, item: &QuoteMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QuoteMetric> {
        self.inner.list()
    }
}

pub fn new_cached_quote_metric_store(
    inner: Rc<dyn QuoteMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QuoteMetricStore> {
    Rc::new(CachedQuoteMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QuoteMetric>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects quote metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMetricQuery {
    pub scope: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QuoteMetricQuery {
    pub fn matches(&self, item: &QuoteMetric) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[QuoteMetric]) -> Vec<QuoteMetric> {
        let matched = collections::filter(items, |candidate: &QuoteMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_quote_metrics(store: Rc<dyn QuoteMetricStore>, query: &QuoteMetricQuery) -> Vec<QuoteMetric> {
    query.apply(&store.list())
}
