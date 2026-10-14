use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped rollout audits by code.
pub trait CappedRolloutAuditStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedRolloutAudit>;
    fn save(&self, item: &CappedRolloutAudit);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedRolloutAudit>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedRolloutAuditStore {
    pub items: RefCell<BTreeMap<String, CappedRolloutAudit>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedRolloutAuditStore for MemoryCappedRolloutAuditStore {
    fn find(&self, code: &str) -> Option<CappedRolloutAudit> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedRolloutAudit) {
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

    fn list(&self) -> Vec<CappedRolloutAudit> {
        let mut out: Vec<CappedRolloutAudit> = Vec::new();
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

pub fn new_memory_capped_rollout_audit_store() -> Rc<dyn CappedRolloutAuditStore> {
    Rc::new(MemoryCappedRolloutAuditStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedRolloutAuditStore {
    pub inner: Rc<dyn CappedRolloutAuditStore>,
    pub cache: RefCell<cache::TtlCache<CappedRolloutAudit>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedRolloutAuditStore for CachedCappedRolloutAuditStore {
    fn find(&self, code: &str) -> Option<CappedRolloutAudit> {
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

    fn save(&self, item: &CappedRolloutAudit) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedRolloutAudit> {
        self.inner.list()
    }
}

pub fn new_cached_capped_rollout_audit_store(
    inner: Rc<dyn CappedRolloutAuditStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedRolloutAuditStore> {
    Rc::new(CachedCappedRolloutAuditStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedRolloutAudit>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped rollout audits. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRolloutAuditQuery {
    pub name: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedRolloutAuditQuery {
    pub fn matches(&self, item: &CappedRolloutAudit) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedRolloutAudit]) -> Vec<CappedRolloutAudit> {
        let matched = collections::filter(items, |candidate: &CappedRolloutAudit| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_rollout_audits(
    store: Rc<dyn CappedRolloutAuditStore>,
    query: &CappedRolloutAuditQuery,
) -> Vec<CappedRolloutAudit> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedRolloutAuditStore> = new_memory_capped_rollout_audit_store();
        let item = new_capped_rollout_audit("code-923", "name-794");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
