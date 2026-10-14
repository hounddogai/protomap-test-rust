use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps health check groups by code.
pub trait HealthCheckGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<HealthCheckGroup>;
    fn save(&self, item: &HealthCheckGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<HealthCheckGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryHealthCheckGroupStore {
    pub items: RefCell<BTreeMap<String, HealthCheckGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl HealthCheckGroupStore for MemoryHealthCheckGroupStore {
    fn find(&self, code: &str) -> Option<HealthCheckGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &HealthCheckGroup) {
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

    fn list(&self) -> Vec<HealthCheckGroup> {
        let mut out: Vec<HealthCheckGroup> = Vec::new();
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

pub fn new_memory_health_check_group_store() -> Rc<dyn HealthCheckGroupStore> {
    Rc::new(MemoryHealthCheckGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedHealthCheckGroupStore {
    pub inner: Rc<dyn HealthCheckGroupStore>,
    pub cache: RefCell<cache::TtlCache<HealthCheckGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl HealthCheckGroupStore for CachedHealthCheckGroupStore {
    fn find(&self, code: &str) -> Option<HealthCheckGroup> {
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

    fn save(&self, item: &HealthCheckGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<HealthCheckGroup> {
        self.inner.list()
    }
}

pub fn new_cached_health_check_group_store(
    inner: Rc<dyn HealthCheckGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn HealthCheckGroupStore> {
    Rc::new(CachedHealthCheckGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<HealthCheckGroup>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects health check groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckGroupQuery {
    pub name: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl HealthCheckGroupQuery {
    pub fn matches(&self, item: &HealthCheckGroup) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[HealthCheckGroup]) -> Vec<HealthCheckGroup> {
        let matched = collections::filter(items, |candidate: &HealthCheckGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_health_check_groups(
    store: Rc<dyn HealthCheckGroupStore>,
    query: &HealthCheckGroupQuery,
) -> Vec<HealthCheckGroup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn HealthCheckGroupStore> = new_memory_health_check_group_store();
        let item = new_health_check_group("code-325", "name-231");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
