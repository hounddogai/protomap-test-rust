use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rollback groups by code.
pub trait RollbackGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollbackGroup>;
    fn save(&self, item: &RollbackGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollbackGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollbackGroupStore {
    pub items: RefCell<BTreeMap<String, RollbackGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl RollbackGroupStore for MemoryRollbackGroupStore {
    fn find(&self, code: &str) -> Option<RollbackGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollbackGroup) {
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

    fn list(&self) -> Vec<RollbackGroup> {
        let mut out: Vec<RollbackGroup> = Vec::new();
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

pub fn new_memory_rollback_group_store() -> Rc<dyn RollbackGroupStore> {
    Rc::new(MemoryRollbackGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollbackGroupStore {
    pub inner: Rc<dyn RollbackGroupStore>,
    pub cache: RefCell<cache::TtlCache<RollbackGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollbackGroupStore for CachedRollbackGroupStore {
    fn find(&self, code: &str) -> Option<RollbackGroup> {
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

    fn save(&self, item: &RollbackGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollbackGroup> {
        self.inner.list()
    }
}

pub fn new_cached_rollback_group_store(
    inner: Rc<dyn RollbackGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollbackGroupStore> {
    Rc::new(CachedRollbackGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollbackGroup>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rollback groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackGroupQuery {
    pub name: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollbackGroupQuery {
    pub fn matches(&self, item: &RollbackGroup) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollbackGroup]) -> Vec<RollbackGroup> {
        let matched = collections::filter(items, |candidate: &RollbackGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rollback_groups(store: Rc<dyn RollbackGroupStore>, query: &RollbackGroupQuery) -> Vec<RollbackGroup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollbackGroupStore> = new_memory_rollback_group_store();
        let item = new_rollback_group("code-438", "name-587");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
