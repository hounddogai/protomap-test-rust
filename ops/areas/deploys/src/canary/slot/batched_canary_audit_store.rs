use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched canary audits by code.
pub trait BatchedCanaryAuditStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedCanaryAudit>;
    fn save(&self, item: &BatchedCanaryAudit);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedCanaryAudit>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedCanaryAuditStore {
    pub items: RefCell<BTreeMap<String, BatchedCanaryAudit>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedCanaryAuditStore for MemoryBatchedCanaryAuditStore {
    fn find(&self, code: &str) -> Option<BatchedCanaryAudit> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedCanaryAudit) {
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

    fn list(&self) -> Vec<BatchedCanaryAudit> {
        let mut out: Vec<BatchedCanaryAudit> = Vec::new();
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

pub fn new_memory_batched_canary_audit_store() -> Rc<dyn BatchedCanaryAuditStore> {
    Rc::new(MemoryBatchedCanaryAuditStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedCanaryAuditStore {
    pub inner: Rc<dyn BatchedCanaryAuditStore>,
    pub cache: RefCell<cache::TtlCache<BatchedCanaryAudit>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedCanaryAuditStore for CachedBatchedCanaryAuditStore {
    fn find(&self, code: &str) -> Option<BatchedCanaryAudit> {
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

    fn save(&self, item: &BatchedCanaryAudit) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedCanaryAudit> {
        self.inner.list()
    }
}

pub fn new_cached_batched_canary_audit_store(
    inner: Rc<dyn BatchedCanaryAuditStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedCanaryAuditStore> {
    Rc::new(CachedBatchedCanaryAuditStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedCanaryAudit>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched canary audits. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCanaryAuditQuery {
    pub unit: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedCanaryAuditQuery {
    pub fn matches(&self, item: &BatchedCanaryAudit) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedCanaryAudit]) -> Vec<BatchedCanaryAudit> {
        let matched = collections::filter(items, |candidate: &BatchedCanaryAudit| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_canary_audits(
    store: Rc<dyn BatchedCanaryAuditStore>,
    query: &BatchedCanaryAuditQuery,
) -> Vec<BatchedCanaryAudit> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedCanaryAuditStore> = new_memory_batched_canary_audit_store();
        let item = new_batched_canary_audit("code-862", "label-600");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
