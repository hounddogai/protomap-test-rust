use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual approval preferences by code.
pub trait ManualApprovalPreferenceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualApprovalPreference>;
    fn save(&self, item: &ManualApprovalPreference);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualApprovalPreference>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualApprovalPreferenceStore {
    pub items: RefCell<BTreeMap<String, ManualApprovalPreference>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualApprovalPreferenceStore for MemoryManualApprovalPreferenceStore {
    fn find(&self, code: &str) -> Option<ManualApprovalPreference> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualApprovalPreference) {
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

    fn list(&self) -> Vec<ManualApprovalPreference> {
        let mut out: Vec<ManualApprovalPreference> = Vec::new();
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

pub fn new_memory_manual_approval_preference_store() -> Rc<dyn ManualApprovalPreferenceStore> {
    Rc::new(MemoryManualApprovalPreferenceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualApprovalPreferenceStore {
    pub inner: Rc<dyn ManualApprovalPreferenceStore>,
    pub cache: RefCell<cache::TtlCache<ManualApprovalPreference>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualApprovalPreferenceStore for CachedManualApprovalPreferenceStore {
    fn find(&self, code: &str) -> Option<ManualApprovalPreference> {
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

    fn save(&self, item: &ManualApprovalPreference) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualApprovalPreference> {
        self.inner.list()
    }
}

pub fn new_cached_manual_approval_preference_store(
    inner: Rc<dyn ManualApprovalPreferenceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualApprovalPreferenceStore> {
    Rc::new(CachedManualApprovalPreferenceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualApprovalPreference>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual approval preferences. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualApprovalPreferenceQuery {
    pub title: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualApprovalPreferenceQuery {
    pub fn matches(&self, item: &ManualApprovalPreference) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualApprovalPreference]) -> Vec<ManualApprovalPreference> {
        let matched = collections::filter(items, |candidate: &ManualApprovalPreference| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_approval_preferences(
    store: Rc<dyn ManualApprovalPreferenceStore>,
    query: &ManualApprovalPreferenceQuery,
) -> Vec<ManualApprovalPreference> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualApprovalPreferenceStore> = new_memory_manual_approval_preference_store();
        let item = new_manual_approval_preference("code-554", "title-22");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
