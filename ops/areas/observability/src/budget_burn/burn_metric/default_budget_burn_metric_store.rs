use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default budget burn metrics by code.
pub trait DefaultBudgetBurnMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultBudgetBurnMetric>;
    fn save(&self, item: &DefaultBudgetBurnMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultBudgetBurnMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultBudgetBurnMetricStore {
    pub items: RefCell<BTreeMap<String, DefaultBudgetBurnMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultBudgetBurnMetricStore for MemoryDefaultBudgetBurnMetricStore {
    fn find(&self, code: &str) -> Option<DefaultBudgetBurnMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultBudgetBurnMetric) {
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

    fn list(&self) -> Vec<DefaultBudgetBurnMetric> {
        let mut out: Vec<DefaultBudgetBurnMetric> = Vec::new();
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

pub fn new_memory_default_budget_burn_metric_store() -> Rc<dyn DefaultBudgetBurnMetricStore> {
    Rc::new(MemoryDefaultBudgetBurnMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultBudgetBurnMetricStore {
    pub inner: Rc<dyn DefaultBudgetBurnMetricStore>,
    pub cache: RefCell<cache::TtlCache<DefaultBudgetBurnMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultBudgetBurnMetricStore for CachedDefaultBudgetBurnMetricStore {
    fn find(&self, code: &str) -> Option<DefaultBudgetBurnMetric> {
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

    fn save(&self, item: &DefaultBudgetBurnMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultBudgetBurnMetric> {
        self.inner.list()
    }
}

pub fn new_cached_default_budget_burn_metric_store(
    inner: Rc<dyn DefaultBudgetBurnMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultBudgetBurnMetricStore> {
    Rc::new(CachedDefaultBudgetBurnMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultBudgetBurnMetric>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default budget burn metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultBudgetBurnMetricQuery {
    pub label: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultBudgetBurnMetricQuery {
    pub fn matches(&self, item: &DefaultBudgetBurnMetric) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultBudgetBurnMetric]) -> Vec<DefaultBudgetBurnMetric> {
        let matched = collections::filter(items, |candidate: &DefaultBudgetBurnMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_budget_burn_metrics(
    store: Rc<dyn DefaultBudgetBurnMetricStore>,
    query: &DefaultBudgetBurnMetricQuery,
) -> Vec<DefaultBudgetBurnMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DefaultBudgetBurnMetricStore> = new_memory_default_budget_burn_metric_store();
        let item = new_default_budget_burn_metric("code-117", "label-907");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
