use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps health check cycles by code.
pub trait HealthCheckCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<HealthCheckCycle>;
    fn save(&self, item: &HealthCheckCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<HealthCheckCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryHealthCheckCycleStore {
    pub items: RefCell<BTreeMap<String, HealthCheckCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl HealthCheckCycleStore for MemoryHealthCheckCycleStore {
    fn find(&self, code: &str) -> Option<HealthCheckCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &HealthCheckCycle) {
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

    fn list(&self) -> Vec<HealthCheckCycle> {
        let mut out: Vec<HealthCheckCycle> = Vec::new();
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

pub fn new_memory_health_check_cycle_store() -> Rc<dyn HealthCheckCycleStore> {
    Rc::new(MemoryHealthCheckCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedHealthCheckCycleStore {
    pub inner: Rc<dyn HealthCheckCycleStore>,
    pub cache: RefCell<cache::TtlCache<HealthCheckCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl HealthCheckCycleStore for CachedHealthCheckCycleStore {
    fn find(&self, code: &str) -> Option<HealthCheckCycle> {
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

    fn save(&self, item: &HealthCheckCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<HealthCheckCycle> {
        self.inner.list()
    }
}

pub fn new_cached_health_check_cycle_store(
    inner: Rc<dyn HealthCheckCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn HealthCheckCycleStore> {
    Rc::new(CachedHealthCheckCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<HealthCheckCycle>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects health check cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckCycleQuery {
    pub label: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl HealthCheckCycleQuery {
    pub fn matches(&self, item: &HealthCheckCycle) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[HealthCheckCycle]) -> Vec<HealthCheckCycle> {
        let matched = collections::filter(items, |candidate: &HealthCheckCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_health_check_cycles(
    store: Rc<dyn HealthCheckCycleStore>,
    query: &HealthCheckCycleQuery,
) -> Vec<HealthCheckCycle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn HealthCheckCycleStore> = new_memory_health_check_cycle_store();
        let item = new_health_check_cycle("code-302", "label-61");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
