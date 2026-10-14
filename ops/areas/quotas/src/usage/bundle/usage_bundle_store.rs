use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps usage bundles by code.
pub trait UsageBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<UsageBundle>;
    fn save(&self, item: &UsageBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<UsageBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryUsageBundleStore {
    pub items: RefCell<BTreeMap<String, UsageBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl UsageBundleStore for MemoryUsageBundleStore {
    fn find(&self, code: &str) -> Option<UsageBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &UsageBundle) {
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

    fn list(&self) -> Vec<UsageBundle> {
        let mut out: Vec<UsageBundle> = Vec::new();
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

pub fn new_memory_usage_bundle_store() -> Rc<dyn UsageBundleStore> {
    Rc::new(MemoryUsageBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedUsageBundleStore {
    pub inner: Rc<dyn UsageBundleStore>,
    pub cache: RefCell<cache::TtlCache<UsageBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl UsageBundleStore for CachedUsageBundleStore {
    fn find(&self, code: &str) -> Option<UsageBundle> {
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

    fn save(&self, item: &UsageBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<UsageBundle> {
        self.inner.list()
    }
}

pub fn new_cached_usage_bundle_store(
    inner: Rc<dyn UsageBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn UsageBundleStore> {
    Rc::new(CachedUsageBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<UsageBundle>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects usage bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageBundleQuery {
    pub region: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl UsageBundleQuery {
    pub fn matches(&self, item: &UsageBundle) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[UsageBundle]) -> Vec<UsageBundle> {
        let matched = collections::filter(items, |candidate: &UsageBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_usage_bundles(store: Rc<dyn UsageBundleStore>, query: &UsageBundleQuery) -> Vec<UsageBundle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn UsageBundleStore> = new_memory_usage_bundle_store();
        let item = new_usage_bundle("code-817", "name-115");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
