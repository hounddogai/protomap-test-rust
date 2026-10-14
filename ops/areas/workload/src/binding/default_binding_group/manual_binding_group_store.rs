use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual binding groups by code.
pub trait ManualBindingGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualBindingGroup>;
    fn save(&self, item: &ManualBindingGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualBindingGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualBindingGroupStore {
    pub items: RefCell<BTreeMap<String, ManualBindingGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualBindingGroupStore for MemoryManualBindingGroupStore {
    fn find(&self, code: &str) -> Option<ManualBindingGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualBindingGroup) {
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

    fn list(&self) -> Vec<ManualBindingGroup> {
        let mut out: Vec<ManualBindingGroup> = Vec::new();
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

pub fn new_memory_manual_binding_group_store() -> Rc<dyn ManualBindingGroupStore> {
    Rc::new(MemoryManualBindingGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualBindingGroupStore {
    pub inner: Rc<dyn ManualBindingGroupStore>,
    pub cache: RefCell<cache::TtlCache<ManualBindingGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualBindingGroupStore for CachedManualBindingGroupStore {
    fn find(&self, code: &str) -> Option<ManualBindingGroup> {
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

    fn save(&self, item: &ManualBindingGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualBindingGroup> {
        self.inner.list()
    }
}

pub fn new_cached_manual_binding_group_store(
    inner: Rc<dyn ManualBindingGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualBindingGroupStore> {
    Rc::new(CachedManualBindingGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualBindingGroup>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual binding groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBindingGroupQuery {
    pub name: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualBindingGroupQuery {
    pub fn matches(&self, item: &ManualBindingGroup) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualBindingGroup]) -> Vec<ManualBindingGroup> {
        let matched = collections::filter(items, |candidate: &ManualBindingGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_binding_groups(
    store: Rc<dyn ManualBindingGroupStore>,
    query: &ManualBindingGroupQuery,
) -> Vec<ManualBindingGroup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualBindingGroupStore> = new_memory_manual_binding_group_store();
        let item = new_manual_binding_group("code-477", "name-247");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
