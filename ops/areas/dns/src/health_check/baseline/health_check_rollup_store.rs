use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps health check rollups by code.
pub trait HealthCheckRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<HealthCheckRollup>;
    fn save(&self, item: &HealthCheckRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<HealthCheckRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryHealthCheckRollupStore {
    pub items: RefCell<BTreeMap<String, HealthCheckRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl HealthCheckRollupStore for MemoryHealthCheckRollupStore {
    fn find(&self, code: &str) -> Option<HealthCheckRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &HealthCheckRollup) {
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

    fn list(&self) -> Vec<HealthCheckRollup> {
        let mut out: Vec<HealthCheckRollup> = Vec::new();
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

pub fn new_memory_health_check_rollup_store() -> Rc<dyn HealthCheckRollupStore> {
    Rc::new(MemoryHealthCheckRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedHealthCheckRollupStore {
    pub inner: Rc<dyn HealthCheckRollupStore>,
    pub cache: RefCell<cache::TtlCache<HealthCheckRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl HealthCheckRollupStore for CachedHealthCheckRollupStore {
    fn find(&self, code: &str) -> Option<HealthCheckRollup> {
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

    fn save(&self, item: &HealthCheckRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<HealthCheckRollup> {
        self.inner.list()
    }
}

pub fn new_cached_health_check_rollup_store(
    inner: Rc<dyn HealthCheckRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn HealthCheckRollupStore> {
    Rc::new(CachedHealthCheckRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<HealthCheckRollup>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects health check rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRollupQuery {
    pub name: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl HealthCheckRollupQuery {
    pub fn matches(&self, item: &HealthCheckRollup) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[HealthCheckRollup]) -> Vec<HealthCheckRollup> {
        let matched = collections::filter(items, |candidate: &HealthCheckRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_health_check_rollups(
    store: Rc<dyn HealthCheckRollupStore>,
    query: &HealthCheckRollupQuery,
) -> Vec<HealthCheckRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn HealthCheckRollupStore> = new_memory_health_check_rollup_store();
        let item = new_health_check_rollup("code-828", "name-628");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
