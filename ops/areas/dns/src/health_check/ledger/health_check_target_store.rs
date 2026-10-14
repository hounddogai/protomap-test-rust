use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps health check targets by code.
pub trait HealthCheckTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<HealthCheckTarget>;
    fn save(&self, item: &HealthCheckTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<HealthCheckTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryHealthCheckTargetStore {
    pub items: RefCell<BTreeMap<String, HealthCheckTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl HealthCheckTargetStore for MemoryHealthCheckTargetStore {
    fn find(&self, code: &str) -> Option<HealthCheckTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &HealthCheckTarget) {
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

    fn list(&self) -> Vec<HealthCheckTarget> {
        let mut out: Vec<HealthCheckTarget> = Vec::new();
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

pub fn new_memory_health_check_target_store() -> Rc<dyn HealthCheckTargetStore> {
    Rc::new(MemoryHealthCheckTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedHealthCheckTargetStore {
    pub inner: Rc<dyn HealthCheckTargetStore>,
    pub cache: RefCell<cache::TtlCache<HealthCheckTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl HealthCheckTargetStore for CachedHealthCheckTargetStore {
    fn find(&self, code: &str) -> Option<HealthCheckTarget> {
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

    fn save(&self, item: &HealthCheckTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<HealthCheckTarget> {
        self.inner.list()
    }
}

pub fn new_cached_health_check_target_store(
    inner: Rc<dyn HealthCheckTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn HealthCheckTargetStore> {
    Rc::new(CachedHealthCheckTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<HealthCheckTarget>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects health check targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTargetQuery {
    pub cluster: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl HealthCheckTargetQuery {
    pub fn matches(&self, item: &HealthCheckTarget) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[HealthCheckTarget]) -> Vec<HealthCheckTarget> {
        let matched = collections::filter(items, |candidate: &HealthCheckTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_health_check_targets(
    store: Rc<dyn HealthCheckTargetStore>,
    query: &HealthCheckTargetQuery,
) -> Vec<HealthCheckTarget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn HealthCheckTargetStore> = new_memory_health_check_target_store();
        let item = new_health_check_target("code-123", "title-775");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
