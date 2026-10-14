use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global audit event rollups by code.
pub trait GlobalAuditEventRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalAuditEventRollup>;
    fn save(&self, item: &GlobalAuditEventRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalAuditEventRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalAuditEventRollupStore {
    pub items: RefCell<BTreeMap<String, GlobalAuditEventRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalAuditEventRollupStore for MemoryGlobalAuditEventRollupStore {
    fn find(&self, code: &str) -> Option<GlobalAuditEventRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalAuditEventRollup) {
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

    fn list(&self) -> Vec<GlobalAuditEventRollup> {
        let mut out: Vec<GlobalAuditEventRollup> = Vec::new();
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

pub fn new_memory_global_audit_event_rollup_store() -> Rc<dyn GlobalAuditEventRollupStore> {
    Rc::new(MemoryGlobalAuditEventRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalAuditEventRollupStore {
    pub inner: Rc<dyn GlobalAuditEventRollupStore>,
    pub cache: RefCell<cache::TtlCache<GlobalAuditEventRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalAuditEventRollupStore for CachedGlobalAuditEventRollupStore {
    fn find(&self, code: &str) -> Option<GlobalAuditEventRollup> {
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

    fn save(&self, item: &GlobalAuditEventRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalAuditEventRollup> {
        self.inner.list()
    }
}

pub fn new_cached_global_audit_event_rollup_store(
    inner: Rc<dyn GlobalAuditEventRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalAuditEventRollupStore> {
    Rc::new(CachedGlobalAuditEventRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalAuditEventRollup>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global audit event rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventRollupQuery {
    pub scope: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalAuditEventRollupQuery {
    pub fn matches(&self, item: &GlobalAuditEventRollup) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
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

    pub fn apply(&self, items: &[GlobalAuditEventRollup]) -> Vec<GlobalAuditEventRollup> {
        let matched = collections::filter(items, |candidate: &GlobalAuditEventRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_audit_event_rollups(
    store: Rc<dyn GlobalAuditEventRollupStore>,
    query: &GlobalAuditEventRollupQuery,
) -> Vec<GlobalAuditEventRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalAuditEventRollupStore> = new_memory_global_audit_event_rollup_store();
        let item = new_global_audit_event_rollup("code-513", "label-193");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
