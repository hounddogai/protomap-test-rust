use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps approval thresholds by code.
pub trait ApprovalThresholdStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ApprovalThreshold>;
    fn save(&self, item: &ApprovalThreshold);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ApprovalThreshold>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryApprovalThresholdStore {
    pub items: RefCell<BTreeMap<String, ApprovalThreshold>>,
    pub order: RefCell<Vec<String>>,
}

impl ApprovalThresholdStore for MemoryApprovalThresholdStore {
    fn find(&self, code: &str) -> Option<ApprovalThreshold> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ApprovalThreshold) {
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

    fn list(&self) -> Vec<ApprovalThreshold> {
        let mut out: Vec<ApprovalThreshold> = Vec::new();
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

pub fn new_memory_approval_threshold_store() -> Rc<dyn ApprovalThresholdStore> {
    Rc::new(MemoryApprovalThresholdStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedApprovalThresholdStore {
    pub inner: Rc<dyn ApprovalThresholdStore>,
    pub cache: RefCell<cache::TtlCache<ApprovalThreshold>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ApprovalThresholdStore for CachedApprovalThresholdStore {
    fn find(&self, code: &str) -> Option<ApprovalThreshold> {
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

    fn save(&self, item: &ApprovalThreshold) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ApprovalThreshold> {
        self.inner.list()
    }
}

pub fn new_cached_approval_threshold_store(
    inner: Rc<dyn ApprovalThresholdStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ApprovalThresholdStore> {
    Rc::new(CachedApprovalThresholdStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ApprovalThreshold>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects approval thresholds. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalThresholdQuery {
    pub label: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ApprovalThresholdQuery {
    pub fn matches(&self, item: &ApprovalThreshold) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[ApprovalThreshold]) -> Vec<ApprovalThreshold> {
        let matched = collections::filter(items, |candidate: &ApprovalThreshold| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_approval_thresholds(
    store: Rc<dyn ApprovalThresholdStore>,
    query: &ApprovalThresholdQuery,
) -> Vec<ApprovalThreshold> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ApprovalThresholdStore> = new_memory_approval_threshold_store();
        let item = new_approval_threshold("code-719", "label-102");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
