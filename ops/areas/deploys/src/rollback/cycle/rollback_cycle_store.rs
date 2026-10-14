use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rollback cycles by code.
pub trait RollbackCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollbackCycle>;
    fn save(&self, item: &RollbackCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollbackCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollbackCycleStore {
    pub items: RefCell<BTreeMap<String, RollbackCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl RollbackCycleStore for MemoryRollbackCycleStore {
    fn find(&self, code: &str) -> Option<RollbackCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollbackCycle) {
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

    fn list(&self) -> Vec<RollbackCycle> {
        let mut out: Vec<RollbackCycle> = Vec::new();
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

pub fn new_memory_rollback_cycle_store() -> Rc<dyn RollbackCycleStore> {
    Rc::new(MemoryRollbackCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollbackCycleStore {
    pub inner: Rc<dyn RollbackCycleStore>,
    pub cache: RefCell<cache::TtlCache<RollbackCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollbackCycleStore for CachedRollbackCycleStore {
    fn find(&self, code: &str) -> Option<RollbackCycle> {
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

    fn save(&self, item: &RollbackCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollbackCycle> {
        self.inner.list()
    }
}

pub fn new_cached_rollback_cycle_store(
    inner: Rc<dyn RollbackCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollbackCycleStore> {
    Rc::new(CachedRollbackCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollbackCycle>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rollback cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackCycleQuery {
    pub cluster: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollbackCycleQuery {
    pub fn matches(&self, item: &RollbackCycle) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollbackCycle]) -> Vec<RollbackCycle> {
        let matched = collections::filter(items, |candidate: &RollbackCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rollback_cycles(store: Rc<dyn RollbackCycleStore>, query: &RollbackCycleQuery) -> Vec<RollbackCycle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollbackCycleStore> = new_memory_rollback_cycle_store();
        let item = new_rollback_cycle("code-900", "name-28");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
