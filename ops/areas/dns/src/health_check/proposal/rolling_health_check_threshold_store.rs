use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling health check thresholds by code.
pub trait RollingHealthCheckThresholdStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingHealthCheckThreshold>;
    fn save(&self, item: &RollingHealthCheckThreshold);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingHealthCheckThreshold>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingHealthCheckThresholdStore {
    pub items: RefCell<BTreeMap<String, RollingHealthCheckThreshold>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingHealthCheckThresholdStore for MemoryRollingHealthCheckThresholdStore {
    fn find(&self, code: &str) -> Option<RollingHealthCheckThreshold> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingHealthCheckThreshold) {
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

    fn list(&self) -> Vec<RollingHealthCheckThreshold> {
        let mut out: Vec<RollingHealthCheckThreshold> = Vec::new();
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

pub fn new_memory_rolling_health_check_threshold_store() -> Rc<dyn RollingHealthCheckThresholdStore> {
    Rc::new(MemoryRollingHealthCheckThresholdStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingHealthCheckThresholdStore {
    pub inner: Rc<dyn RollingHealthCheckThresholdStore>,
    pub cache: RefCell<cache::TtlCache<RollingHealthCheckThreshold>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingHealthCheckThresholdStore for CachedRollingHealthCheckThresholdStore {
    fn find(&self, code: &str) -> Option<RollingHealthCheckThreshold> {
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

    fn save(&self, item: &RollingHealthCheckThreshold) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingHealthCheckThreshold> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_health_check_threshold_store(
    inner: Rc<dyn RollingHealthCheckThresholdStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingHealthCheckThresholdStore> {
    Rc::new(CachedRollingHealthCheckThresholdStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingHealthCheckThreshold>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling health check thresholds. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingHealthCheckThresholdQuery {
    pub title: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingHealthCheckThresholdQuery {
    pub fn matches(&self, item: &RollingHealthCheckThreshold) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingHealthCheckThreshold]) -> Vec<RollingHealthCheckThreshold> {
        let matched = collections::filter(items, |candidate: &RollingHealthCheckThreshold| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_health_check_thresholds(
    store: Rc<dyn RollingHealthCheckThresholdStore>,
    query: &RollingHealthCheckThresholdQuery,
) -> Vec<RollingHealthCheckThreshold> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingHealthCheckThresholdStore> = new_memory_rolling_health_check_threshold_store();
        let item = new_rolling_health_check_threshold("code-992", "title-244");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
