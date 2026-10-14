use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual audit event metrics by code.
pub trait ManualAuditEventMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualAuditEventMetric>;
    fn save(&self, item: &ManualAuditEventMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualAuditEventMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualAuditEventMetricStore {
    pub items: RefCell<BTreeMap<String, ManualAuditEventMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualAuditEventMetricStore for MemoryManualAuditEventMetricStore {
    fn find(&self, code: &str) -> Option<ManualAuditEventMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualAuditEventMetric) {
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

    fn list(&self) -> Vec<ManualAuditEventMetric> {
        let mut out: Vec<ManualAuditEventMetric> = Vec::new();
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

pub fn new_memory_manual_audit_event_metric_store() -> Rc<dyn ManualAuditEventMetricStore> {
    Rc::new(MemoryManualAuditEventMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualAuditEventMetricStore {
    pub inner: Rc<dyn ManualAuditEventMetricStore>,
    pub cache: RefCell<cache::TtlCache<ManualAuditEventMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualAuditEventMetricStore for CachedManualAuditEventMetricStore {
    fn find(&self, code: &str) -> Option<ManualAuditEventMetric> {
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

    fn save(&self, item: &ManualAuditEventMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualAuditEventMetric> {
        self.inner.list()
    }
}

pub fn new_cached_manual_audit_event_metric_store(
    inner: Rc<dyn ManualAuditEventMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualAuditEventMetricStore> {
    Rc::new(CachedManualAuditEventMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualAuditEventMetric>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual audit event metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventMetricQuery {
    pub region: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualAuditEventMetricQuery {
    pub fn matches(&self, item: &ManualAuditEventMetric) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualAuditEventMetric]) -> Vec<ManualAuditEventMetric> {
        let matched = collections::filter(items, |candidate: &ManualAuditEventMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_audit_event_metrics(
    store: Rc<dyn ManualAuditEventMetricStore>,
    query: &ManualAuditEventMetricQuery,
) -> Vec<ManualAuditEventMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualAuditEventMetricStore> = new_memory_manual_audit_event_metric_store();
        let item = new_manual_audit_event_metric("code-901", "label-94");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
