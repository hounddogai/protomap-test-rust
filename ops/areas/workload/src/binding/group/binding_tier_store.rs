use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps binding tiers by code.
pub trait BindingTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BindingTier>;
    fn save(&self, item: &BindingTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BindingTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBindingTierStore {
    pub items: RefCell<BTreeMap<String, BindingTier>>,
    pub order: RefCell<Vec<String>>,
}

impl BindingTierStore for MemoryBindingTierStore {
    fn find(&self, code: &str) -> Option<BindingTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BindingTier) {
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

    fn list(&self) -> Vec<BindingTier> {
        let mut out: Vec<BindingTier> = Vec::new();
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

pub fn new_memory_binding_tier_store() -> Rc<dyn BindingTierStore> {
    Rc::new(MemoryBindingTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBindingTierStore {
    pub inner: Rc<dyn BindingTierStore>,
    pub cache: RefCell<cache::TtlCache<BindingTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BindingTierStore for CachedBindingTierStore {
    fn find(&self, code: &str) -> Option<BindingTier> {
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

    fn save(&self, item: &BindingTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BindingTier> {
        self.inner.list()
    }
}

pub fn new_cached_binding_tier_store(
    inner: Rc<dyn BindingTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BindingTierStore> {
    Rc::new(CachedBindingTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BindingTier>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects binding tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingTierQuery {
    pub segment: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BindingTierQuery {
    pub fn matches(&self, item: &BindingTier) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BindingTier]) -> Vec<BindingTier> {
        let matched = collections::filter(items, |candidate: &BindingTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_binding_tiers(store: Rc<dyn BindingTierStore>, query: &BindingTierQuery) -> Vec<BindingTier> {
    query.apply(&store.list())
}
