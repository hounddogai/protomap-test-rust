use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted budget burn metrics by code.
pub trait WeightedBudgetBurnMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedBudgetBurnMetric>;
    fn save(&self, item: &WeightedBudgetBurnMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedBudgetBurnMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedBudgetBurnMetricStore {
    pub items: RefCell<BTreeMap<String, WeightedBudgetBurnMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedBudgetBurnMetricStore for MemoryWeightedBudgetBurnMetricStore {
    fn find(&self, code: &str) -> Option<WeightedBudgetBurnMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedBudgetBurnMetric) {
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

    fn list(&self) -> Vec<WeightedBudgetBurnMetric> {
        let mut out: Vec<WeightedBudgetBurnMetric> = Vec::new();
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

pub fn new_memory_weighted_budget_burn_metric_store() -> Rc<dyn WeightedBudgetBurnMetricStore> {
    Rc::new(MemoryWeightedBudgetBurnMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedBudgetBurnMetricStore {
    pub inner: Rc<dyn WeightedBudgetBurnMetricStore>,
    pub cache: RefCell<cache::TtlCache<WeightedBudgetBurnMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedBudgetBurnMetricStore for CachedWeightedBudgetBurnMetricStore {
    fn find(&self, code: &str) -> Option<WeightedBudgetBurnMetric> {
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

    fn save(&self, item: &WeightedBudgetBurnMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedBudgetBurnMetric> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_budget_burn_metric_store(
    inner: Rc<dyn WeightedBudgetBurnMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedBudgetBurnMetricStore> {
    Rc::new(CachedWeightedBudgetBurnMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedBudgetBurnMetric>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted budget burn metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnMetricQuery {
    pub title: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedBudgetBurnMetricQuery {
    pub fn matches(&self, item: &WeightedBudgetBurnMetric) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedBudgetBurnMetric]) -> Vec<WeightedBudgetBurnMetric> {
        let matched = collections::filter(items, |candidate: &WeightedBudgetBurnMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_budget_burn_metrics(
    store: Rc<dyn WeightedBudgetBurnMetricStore>,
    query: &WeightedBudgetBurnMetricQuery,
) -> Vec<WeightedBudgetBurnMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedBudgetBurnMetricStore> = new_memory_weighted_budget_burn_metric_store();
        let item = new_weighted_budget_burn_metric("code-628", "title-794");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
