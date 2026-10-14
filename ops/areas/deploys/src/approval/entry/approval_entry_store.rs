use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps approval entries by code.
pub trait ApprovalEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ApprovalEntry>;
    fn save(&self, item: &ApprovalEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ApprovalEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryApprovalEntryStore {
    pub items: RefCell<BTreeMap<String, ApprovalEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl ApprovalEntryStore for MemoryApprovalEntryStore {
    fn find(&self, code: &str) -> Option<ApprovalEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ApprovalEntry) {
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

    fn list(&self) -> Vec<ApprovalEntry> {
        let mut out: Vec<ApprovalEntry> = Vec::new();
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

pub fn new_memory_approval_entry_store() -> Rc<dyn ApprovalEntryStore> {
    Rc::new(MemoryApprovalEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedApprovalEntryStore {
    pub inner: Rc<dyn ApprovalEntryStore>,
    pub cache: RefCell<cache::TtlCache<ApprovalEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ApprovalEntryStore for CachedApprovalEntryStore {
    fn find(&self, code: &str) -> Option<ApprovalEntry> {
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

    fn save(&self, item: &ApprovalEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ApprovalEntry> {
        self.inner.list()
    }
}

pub fn new_cached_approval_entry_store(
    inner: Rc<dyn ApprovalEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ApprovalEntryStore> {
    Rc::new(CachedApprovalEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ApprovalEntry>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects approval entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalEntryQuery {
    pub name: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ApprovalEntryQuery {
    pub fn matches(&self, item: &ApprovalEntry) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ApprovalEntry]) -> Vec<ApprovalEntry> {
        let matched = collections::filter(items, |candidate: &ApprovalEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_approval_entries(store: Rc<dyn ApprovalEntryStore>, query: &ApprovalEntryQuery) -> Vec<ApprovalEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ApprovalEntryStore> = new_memory_approval_entry_store();
        let item = new_approval_entry("code-397", "name-978");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
