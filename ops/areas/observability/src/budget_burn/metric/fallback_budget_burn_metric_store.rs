use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback budget burn metrics by code.
pub trait FallbackBudgetBurnMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackBudgetBurnMetric>;
    fn save(&self, item: &FallbackBudgetBurnMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackBudgetBurnMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackBudgetBurnMetricStore {
    pub items: RefCell<BTreeMap<String, FallbackBudgetBurnMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackBudgetBurnMetricStore for MemoryFallbackBudgetBurnMetricStore {
    fn find(&self, code: &str) -> Option<FallbackBudgetBurnMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackBudgetBurnMetric) {
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

    fn list(&self) -> Vec<FallbackBudgetBurnMetric> {
        let mut out: Vec<FallbackBudgetBurnMetric> = Vec::new();
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

pub fn new_memory_fallback_budget_burn_metric_store() -> Rc<dyn FallbackBudgetBurnMetricStore> {
    Rc::new(MemoryFallbackBudgetBurnMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackBudgetBurnMetricStore {
    pub inner: Rc<dyn FallbackBudgetBurnMetricStore>,
    pub cache: RefCell<cache::TtlCache<FallbackBudgetBurnMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackBudgetBurnMetricStore for CachedFallbackBudgetBurnMetricStore {
    fn find(&self, code: &str) -> Option<FallbackBudgetBurnMetric> {
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

    fn save(&self, item: &FallbackBudgetBurnMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackBudgetBurnMetric> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_budget_burn_metric_store(
    inner: Rc<dyn FallbackBudgetBurnMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackBudgetBurnMetricStore> {
    Rc::new(CachedFallbackBudgetBurnMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackBudgetBurnMetric>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback budget burn metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBudgetBurnMetricQuery {
    pub channel: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackBudgetBurnMetricQuery {
    pub fn matches(&self, item: &FallbackBudgetBurnMetric) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackBudgetBurnMetric]) -> Vec<FallbackBudgetBurnMetric> {
        let matched = collections::filter(items, |candidate: &FallbackBudgetBurnMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_budget_burn_metrics(
    store: Rc<dyn FallbackBudgetBurnMetricStore>,
    query: &FallbackBudgetBurnMetricQuery,
) -> Vec<FallbackBudgetBurnMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackBudgetBurnMetricStore> = new_memory_fallback_budget_burn_metric_store();
        let item = new_fallback_budget_burn_metric("code-362", "name-977");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
