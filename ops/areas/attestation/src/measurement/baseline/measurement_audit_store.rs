use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps measurement audits by code.
pub trait MeasurementAuditStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MeasurementAudit>;
    fn save(&self, item: &MeasurementAudit);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MeasurementAudit>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMeasurementAuditStore {
    pub items: RefCell<BTreeMap<String, MeasurementAudit>>,
    pub order: RefCell<Vec<String>>,
}

impl MeasurementAuditStore for MemoryMeasurementAuditStore {
    fn find(&self, code: &str) -> Option<MeasurementAudit> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MeasurementAudit) {
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

    fn list(&self) -> Vec<MeasurementAudit> {
        let mut out: Vec<MeasurementAudit> = Vec::new();
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

pub fn new_memory_measurement_audit_store() -> Rc<dyn MeasurementAuditStore> {
    Rc::new(MemoryMeasurementAuditStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMeasurementAuditStore {
    pub inner: Rc<dyn MeasurementAuditStore>,
    pub cache: RefCell<cache::TtlCache<MeasurementAudit>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MeasurementAuditStore for CachedMeasurementAuditStore {
    fn find(&self, code: &str) -> Option<MeasurementAudit> {
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

    fn save(&self, item: &MeasurementAudit) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MeasurementAudit> {
        self.inner.list()
    }
}

pub fn new_cached_measurement_audit_store(
    inner: Rc<dyn MeasurementAuditStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MeasurementAuditStore> {
    Rc::new(CachedMeasurementAuditStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MeasurementAudit>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects measurement audits. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAuditQuery {
    pub source: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MeasurementAuditQuery {
    pub fn matches(&self, item: &MeasurementAudit) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MeasurementAudit]) -> Vec<MeasurementAudit> {
        let matched = collections::filter(items, |candidate: &MeasurementAudit| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_measurement_audits(
    store: Rc<dyn MeasurementAuditStore>,
    query: &MeasurementAuditQuery,
) -> Vec<MeasurementAudit> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn MeasurementAuditStore> = new_memory_measurement_audit_store();
        let item = new_measurement_audit("code-99", "name-547");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
