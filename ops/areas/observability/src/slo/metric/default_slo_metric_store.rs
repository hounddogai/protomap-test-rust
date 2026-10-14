use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default slo metrics by code.
pub trait DefaultSloMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultSloMetric>;
    fn save(&self, item: &DefaultSloMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultSloMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultSloMetricStore {
    pub items: RefCell<BTreeMap<String, DefaultSloMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultSloMetricStore for MemoryDefaultSloMetricStore {
    fn find(&self, code: &str) -> Option<DefaultSloMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultSloMetric) {
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

    fn list(&self) -> Vec<DefaultSloMetric> {
        let mut out: Vec<DefaultSloMetric> = Vec::new();
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

pub fn new_memory_default_slo_metric_store() -> Rc<dyn DefaultSloMetricStore> {
    Rc::new(MemoryDefaultSloMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultSloMetricStore {
    pub inner: Rc<dyn DefaultSloMetricStore>,
    pub cache: RefCell<cache::TtlCache<DefaultSloMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultSloMetricStore for CachedDefaultSloMetricStore {
    fn find(&self, code: &str) -> Option<DefaultSloMetric> {
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

    fn save(&self, item: &DefaultSloMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultSloMetric> {
        self.inner.list()
    }
}

pub fn new_cached_default_slo_metric_store(
    inner: Rc<dyn DefaultSloMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultSloMetricStore> {
    Rc::new(CachedDefaultSloMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultSloMetric>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default slo metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSloMetricQuery {
    pub title: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultSloMetricQuery {
    pub fn matches(&self, item: &DefaultSloMetric) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultSloMetric]) -> Vec<DefaultSloMetric> {
        let matched = collections::filter(items, |candidate: &DefaultSloMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_slo_metrics(
    store: Rc<dyn DefaultSloMetricStore>,
    query: &DefaultSloMetricQuery,
) -> Vec<DefaultSloMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DefaultSloMetricStore> = new_memory_default_slo_metric_store();
        let item = new_default_slo_metric("code-392", "title-870");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
