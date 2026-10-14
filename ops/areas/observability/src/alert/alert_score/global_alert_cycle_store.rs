use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global alert cycles by code.
pub trait GlobalAlertCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalAlertCycle>;
    fn save(&self, item: &GlobalAlertCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalAlertCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalAlertCycleStore {
    pub items: RefCell<BTreeMap<String, GlobalAlertCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalAlertCycleStore for MemoryGlobalAlertCycleStore {
    fn find(&self, code: &str) -> Option<GlobalAlertCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalAlertCycle) {
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

    fn list(&self) -> Vec<GlobalAlertCycle> {
        let mut out: Vec<GlobalAlertCycle> = Vec::new();
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

pub fn new_memory_global_alert_cycle_store() -> Rc<dyn GlobalAlertCycleStore> {
    Rc::new(MemoryGlobalAlertCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalAlertCycleStore {
    pub inner: Rc<dyn GlobalAlertCycleStore>,
    pub cache: RefCell<cache::TtlCache<GlobalAlertCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalAlertCycleStore for CachedGlobalAlertCycleStore {
    fn find(&self, code: &str) -> Option<GlobalAlertCycle> {
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

    fn save(&self, item: &GlobalAlertCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalAlertCycle> {
        self.inner.list()
    }
}

pub fn new_cached_global_alert_cycle_store(
    inner: Rc<dyn GlobalAlertCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalAlertCycleStore> {
    Rc::new(CachedGlobalAlertCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalAlertCycle>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global alert cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertCycleQuery {
    pub cluster: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalAlertCycleQuery {
    pub fn matches(&self, item: &GlobalAlertCycle) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[GlobalAlertCycle]) -> Vec<GlobalAlertCycle> {
        let matched = collections::filter(items, |candidate: &GlobalAlertCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_alert_cycles(
    store: Rc<dyn GlobalAlertCycleStore>,
    query: &GlobalAlertCycleQuery,
) -> Vec<GlobalAlertCycle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalAlertCycleStore> = new_memory_global_alert_cycle_store();
        let item = new_global_alert_cycle("code-528", "name-836");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
