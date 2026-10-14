use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps binding groups by code.
pub trait BindingGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BindingGroup>;
    fn save(&self, item: &BindingGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BindingGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBindingGroupStore {
    pub items: RefCell<BTreeMap<String, BindingGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl BindingGroupStore for MemoryBindingGroupStore {
    fn find(&self, code: &str) -> Option<BindingGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BindingGroup) {
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

    fn list(&self) -> Vec<BindingGroup> {
        let mut out: Vec<BindingGroup> = Vec::new();
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

pub fn new_memory_binding_group_store() -> Rc<dyn BindingGroupStore> {
    Rc::new(MemoryBindingGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBindingGroupStore {
    pub inner: Rc<dyn BindingGroupStore>,
    pub cache: RefCell<cache::TtlCache<BindingGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BindingGroupStore for CachedBindingGroupStore {
    fn find(&self, code: &str) -> Option<BindingGroup> {
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

    fn save(&self, item: &BindingGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BindingGroup> {
        self.inner.list()
    }
}

pub fn new_cached_binding_group_store(
    inner: Rc<dyn BindingGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BindingGroupStore> {
    Rc::new(CachedBindingGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BindingGroup>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects binding groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingGroupQuery {
    pub cluster: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BindingGroupQuery {
    pub fn matches(&self, item: &BindingGroup) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BindingGroup]) -> Vec<BindingGroup> {
        let matched = collections::filter(items, |candidate: &BindingGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_binding_groups(store: Rc<dyn BindingGroupStore>, query: &BindingGroupQuery) -> Vec<BindingGroup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BindingGroupStore> = new_memory_binding_group_store();
        let item = new_binding_group("code-407", "label-424");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
