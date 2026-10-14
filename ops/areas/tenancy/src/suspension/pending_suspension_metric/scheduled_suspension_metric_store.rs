use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled suspension metrics by code.
pub trait ScheduledSuspensionMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledSuspensionMetric>;
    fn save(&self, item: &ScheduledSuspensionMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledSuspensionMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledSuspensionMetricStore {
    pub items: RefCell<BTreeMap<String, ScheduledSuspensionMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledSuspensionMetricStore for MemoryScheduledSuspensionMetricStore {
    fn find(&self, code: &str) -> Option<ScheduledSuspensionMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledSuspensionMetric) {
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

    fn list(&self) -> Vec<ScheduledSuspensionMetric> {
        let mut out: Vec<ScheduledSuspensionMetric> = Vec::new();
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

pub fn new_memory_scheduled_suspension_metric_store() -> Rc<dyn ScheduledSuspensionMetricStore> {
    Rc::new(MemoryScheduledSuspensionMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledSuspensionMetricStore {
    pub inner: Rc<dyn ScheduledSuspensionMetricStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledSuspensionMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledSuspensionMetricStore for CachedScheduledSuspensionMetricStore {
    fn find(&self, code: &str) -> Option<ScheduledSuspensionMetric> {
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

    fn save(&self, item: &ScheduledSuspensionMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledSuspensionMetric> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_suspension_metric_store(
    inner: Rc<dyn ScheduledSuspensionMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledSuspensionMetricStore> {
    Rc::new(CachedScheduledSuspensionMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledSuspensionMetric>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled suspension metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionMetricQuery {
    pub label: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledSuspensionMetricQuery {
    pub fn matches(&self, item: &ScheduledSuspensionMetric) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledSuspensionMetric]) -> Vec<ScheduledSuspensionMetric> {
        let matched = collections::filter(items, |candidate: &ScheduledSuspensionMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_suspension_metrics(
    store: Rc<dyn ScheduledSuspensionMetricStore>,
    query: &ScheduledSuspensionMetricQuery,
) -> Vec<ScheduledSuspensionMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ScheduledSuspensionMetricStore> = new_memory_scheduled_suspension_metric_store();
        let item = new_scheduled_suspension_metric("code-816", "label-207");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
