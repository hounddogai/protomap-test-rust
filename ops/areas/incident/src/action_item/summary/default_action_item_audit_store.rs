use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default action item audits by code.
pub trait DefaultActionItemAuditStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultActionItemAudit>;
    fn save(&self, item: &DefaultActionItemAudit);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultActionItemAudit>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultActionItemAuditStore {
    pub items: RefCell<BTreeMap<String, DefaultActionItemAudit>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultActionItemAuditStore for MemoryDefaultActionItemAuditStore {
    fn find(&self, code: &str) -> Option<DefaultActionItemAudit> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultActionItemAudit) {
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

    fn list(&self) -> Vec<DefaultActionItemAudit> {
        let mut out: Vec<DefaultActionItemAudit> = Vec::new();
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

pub fn new_memory_default_action_item_audit_store() -> Rc<dyn DefaultActionItemAuditStore> {
    Rc::new(MemoryDefaultActionItemAuditStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultActionItemAuditStore {
    pub inner: Rc<dyn DefaultActionItemAuditStore>,
    pub cache: RefCell<cache::TtlCache<DefaultActionItemAudit>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultActionItemAuditStore for CachedDefaultActionItemAuditStore {
    fn find(&self, code: &str) -> Option<DefaultActionItemAudit> {
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

    fn save(&self, item: &DefaultActionItemAudit) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultActionItemAudit> {
        self.inner.list()
    }
}

pub fn new_cached_default_action_item_audit_store(
    inner: Rc<dyn DefaultActionItemAuditStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultActionItemAuditStore> {
    Rc::new(CachedDefaultActionItemAuditStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultActionItemAudit>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default action item audits. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemAuditQuery {
    pub name: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultActionItemAuditQuery {
    pub fn matches(&self, item: &DefaultActionItemAudit) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.ends_at < self.min_ends_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultActionItemAudit]) -> Vec<DefaultActionItemAudit> {
        let matched = collections::filter(items, |candidate: &DefaultActionItemAudit| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_action_item_audits(
    store: Rc<dyn DefaultActionItemAuditStore>,
    query: &DefaultActionItemAuditQuery,
) -> Vec<DefaultActionItemAudit> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DefaultActionItemAuditStore> = new_memory_default_action_item_audit_store();
        let item = new_default_action_item_audit("code-523", "name-90");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
