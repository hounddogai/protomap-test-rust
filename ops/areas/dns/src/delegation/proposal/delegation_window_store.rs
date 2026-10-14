use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps delegation windows by code.
pub trait DelegationWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DelegationWindow>;
    fn save(&self, item: &DelegationWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DelegationWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDelegationWindowStore {
    pub items: RefCell<BTreeMap<String, DelegationWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl DelegationWindowStore for MemoryDelegationWindowStore {
    fn find(&self, code: &str) -> Option<DelegationWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DelegationWindow) {
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

    fn list(&self) -> Vec<DelegationWindow> {
        let mut out: Vec<DelegationWindow> = Vec::new();
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

pub fn new_memory_delegation_window_store() -> Rc<dyn DelegationWindowStore> {
    Rc::new(MemoryDelegationWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDelegationWindowStore {
    pub inner: Rc<dyn DelegationWindowStore>,
    pub cache: RefCell<cache::TtlCache<DelegationWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DelegationWindowStore for CachedDelegationWindowStore {
    fn find(&self, code: &str) -> Option<DelegationWindow> {
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

    fn save(&self, item: &DelegationWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DelegationWindow> {
        self.inner.list()
    }
}

pub fn new_cached_delegation_window_store(
    inner: Rc<dyn DelegationWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DelegationWindowStore> {
    Rc::new(CachedDelegationWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DelegationWindow>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects delegation windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationWindowQuery {
    pub unit: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DelegationWindowQuery {
    pub fn matches(&self, item: &DelegationWindow) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DelegationWindow]) -> Vec<DelegationWindow> {
        let matched = collections::filter(items, |candidate: &DelegationWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_delegation_windows(
    store: Rc<dyn DelegationWindowStore>,
    query: &DelegationWindowQuery,
) -> Vec<DelegationWindow> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DelegationWindowStore> = new_memory_delegation_window_store();
        let item = new_delegation_window("code-766", "name-103");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
