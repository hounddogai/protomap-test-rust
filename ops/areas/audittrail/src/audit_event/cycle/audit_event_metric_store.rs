use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps audit event metrics by code.
pub trait AuditEventMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AuditEventMetric>;
    fn save(&self, item: &AuditEventMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AuditEventMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAuditEventMetricStore {
    pub items: RefCell<BTreeMap<String, AuditEventMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl AuditEventMetricStore for MemoryAuditEventMetricStore {
    fn find(&self, code: &str) -> Option<AuditEventMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AuditEventMetric) {
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

    fn list(&self) -> Vec<AuditEventMetric> {
        let mut out: Vec<AuditEventMetric> = Vec::new();
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

pub fn new_memory_audit_event_metric_store() -> Rc<dyn AuditEventMetricStore> {
    Rc::new(MemoryAuditEventMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAuditEventMetricStore {
    pub inner: Rc<dyn AuditEventMetricStore>,
    pub cache: RefCell<cache::TtlCache<AuditEventMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AuditEventMetricStore for CachedAuditEventMetricStore {
    fn find(&self, code: &str) -> Option<AuditEventMetric> {
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

    fn save(&self, item: &AuditEventMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AuditEventMetric> {
        self.inner.list()
    }
}

pub fn new_cached_audit_event_metric_store(
    inner: Rc<dyn AuditEventMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AuditEventMetricStore> {
    Rc::new(CachedAuditEventMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AuditEventMetric>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects audit event metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventMetricQuery {
    pub cluster: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AuditEventMetricQuery {
    pub fn matches(&self, item: &AuditEventMetric) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AuditEventMetric]) -> Vec<AuditEventMetric> {
        let matched = collections::filter(items, |candidate: &AuditEventMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_audit_event_metrics(
    store: Rc<dyn AuditEventMetricStore>,
    query: &AuditEventMetricQuery,
) -> Vec<AuditEventMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AuditEventMetricStore> = new_memory_audit_event_metric_store();
        let item = new_audit_event_metric("code-919", "name-401");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
