use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps binding bundles by code.
pub trait BindingBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BindingBundle>;
    fn save(&self, item: &BindingBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BindingBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBindingBundleStore {
    pub items: RefCell<BTreeMap<String, BindingBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl BindingBundleStore for MemoryBindingBundleStore {
    fn find(&self, code: &str) -> Option<BindingBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BindingBundle) {
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

    fn list(&self) -> Vec<BindingBundle> {
        let mut out: Vec<BindingBundle> = Vec::new();
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

pub fn new_memory_binding_bundle_store() -> Rc<dyn BindingBundleStore> {
    Rc::new(MemoryBindingBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBindingBundleStore {
    pub inner: Rc<dyn BindingBundleStore>,
    pub cache: RefCell<cache::TtlCache<BindingBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BindingBundleStore for CachedBindingBundleStore {
    fn find(&self, code: &str) -> Option<BindingBundle> {
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

    fn save(&self, item: &BindingBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BindingBundle> {
        self.inner.list()
    }
}

pub fn new_cached_binding_bundle_store(
    inner: Rc<dyn BindingBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BindingBundleStore> {
    Rc::new(CachedBindingBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BindingBundle>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects binding bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingBundleQuery {
    pub title: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BindingBundleQuery {
    pub fn matches(&self, item: &BindingBundle) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[BindingBundle]) -> Vec<BindingBundle> {
        let matched = collections::filter(items, |candidate: &BindingBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_binding_bundles(store: Rc<dyn BindingBundleStore>, query: &BindingBundleQuery) -> Vec<BindingBundle> {
    query.apply(&store.list())
}
