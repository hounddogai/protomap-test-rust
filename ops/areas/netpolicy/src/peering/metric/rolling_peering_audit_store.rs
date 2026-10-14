use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling peering audits by code.
pub trait RollingPeeringAuditStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingPeeringAudit>;
    fn save(&self, item: &RollingPeeringAudit);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingPeeringAudit>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingPeeringAuditStore {
    pub items: RefCell<BTreeMap<String, RollingPeeringAudit>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingPeeringAuditStore for MemoryRollingPeeringAuditStore {
    fn find(&self, code: &str) -> Option<RollingPeeringAudit> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingPeeringAudit) {
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

    fn list(&self) -> Vec<RollingPeeringAudit> {
        let mut out: Vec<RollingPeeringAudit> = Vec::new();
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

pub fn new_memory_rolling_peering_audit_store() -> Rc<dyn RollingPeeringAuditStore> {
    Rc::new(MemoryRollingPeeringAuditStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingPeeringAuditStore {
    pub inner: Rc<dyn RollingPeeringAuditStore>,
    pub cache: RefCell<cache::TtlCache<RollingPeeringAudit>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingPeeringAuditStore for CachedRollingPeeringAuditStore {
    fn find(&self, code: &str) -> Option<RollingPeeringAudit> {
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

    fn save(&self, item: &RollingPeeringAudit) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingPeeringAudit> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_peering_audit_store(
    inner: Rc<dyn RollingPeeringAuditStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingPeeringAuditStore> {
    Rc::new(CachedRollingPeeringAuditStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingPeeringAudit>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling peering audits. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringAuditQuery {
    pub channel: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingPeeringAuditQuery {
    pub fn matches(&self, item: &RollingPeeringAudit) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingPeeringAudit]) -> Vec<RollingPeeringAudit> {
        let matched = collections::filter(items, |candidate: &RollingPeeringAudit| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_peering_audits(
    store: Rc<dyn RollingPeeringAuditStore>,
    query: &RollingPeeringAuditQuery,
) -> Vec<RollingPeeringAudit> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingPeeringAuditStore> = new_memory_rolling_peering_audit_store();
        let item = new_rolling_peering_audit("code-148", "name-599");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
