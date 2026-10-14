use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps failover cycles by code.
pub trait FailoverCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FailoverCycle>;
    fn save(&self, item: &FailoverCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FailoverCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFailoverCycleStore {
    pub items: RefCell<BTreeMap<String, FailoverCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl FailoverCycleStore for MemoryFailoverCycleStore {
    fn find(&self, code: &str) -> Option<FailoverCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FailoverCycle) {
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

    fn list(&self) -> Vec<FailoverCycle> {
        let mut out: Vec<FailoverCycle> = Vec::new();
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

pub fn new_memory_failover_cycle_store() -> Rc<dyn FailoverCycleStore> {
    Rc::new(MemoryFailoverCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFailoverCycleStore {
    pub inner: Rc<dyn FailoverCycleStore>,
    pub cache: RefCell<cache::TtlCache<FailoverCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FailoverCycleStore for CachedFailoverCycleStore {
    fn find(&self, code: &str) -> Option<FailoverCycle> {
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

    fn save(&self, item: &FailoverCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FailoverCycle> {
        self.inner.list()
    }
}

pub fn new_cached_failover_cycle_store(
    inner: Rc<dyn FailoverCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FailoverCycleStore> {
    Rc::new(CachedFailoverCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FailoverCycle>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects failover cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverCycleQuery {
    pub label: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FailoverCycleQuery {
    pub fn matches(&self, item: &FailoverCycle) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FailoverCycle]) -> Vec<FailoverCycle> {
        let matched = collections::filter(items, |candidate: &FailoverCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_failover_cycles(store: Rc<dyn FailoverCycleStore>, query: &FailoverCycleQuery) -> Vec<FailoverCycle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FailoverCycleStore> = new_memory_failover_cycle_store();
        let item = new_failover_cycle("code-187", "label-489");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
