use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps isolation overrides by code.
pub trait IsolationOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<IsolationOverride>;
    fn save(&self, item: &IsolationOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<IsolationOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryIsolationOverrideStore {
    pub items: RefCell<BTreeMap<String, IsolationOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl IsolationOverrideStore for MemoryIsolationOverrideStore {
    fn find(&self, code: &str) -> Option<IsolationOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &IsolationOverride) {
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

    fn list(&self) -> Vec<IsolationOverride> {
        let mut out: Vec<IsolationOverride> = Vec::new();
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

pub fn new_memory_isolation_override_store() -> Rc<dyn IsolationOverrideStore> {
    Rc::new(MemoryIsolationOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedIsolationOverrideStore {
    pub inner: Rc<dyn IsolationOverrideStore>,
    pub cache: RefCell<cache::TtlCache<IsolationOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl IsolationOverrideStore for CachedIsolationOverrideStore {
    fn find(&self, code: &str) -> Option<IsolationOverride> {
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

    fn save(&self, item: &IsolationOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<IsolationOverride> {
        self.inner.list()
    }
}

pub fn new_cached_isolation_override_store(
    inner: Rc<dyn IsolationOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn IsolationOverrideStore> {
    Rc::new(CachedIsolationOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<IsolationOverride>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects isolation overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationOverrideQuery {
    pub unit: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl IsolationOverrideQuery {
    pub fn matches(&self, item: &IsolationOverride) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[IsolationOverride]) -> Vec<IsolationOverride> {
        let matched = collections::filter(items, |candidate: &IsolationOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_isolation_overrides(
    store: Rc<dyn IsolationOverrideStore>,
    query: &IsolationOverrideQuery,
) -> Vec<IsolationOverride> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn IsolationOverrideStore> = new_memory_isolation_override_store();
        let item = new_isolation_override("code-738", "title-624");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
