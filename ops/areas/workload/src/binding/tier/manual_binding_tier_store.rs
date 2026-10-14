use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual binding tiers by code.
pub trait ManualBindingTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualBindingTier>;
    fn save(&self, item: &ManualBindingTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualBindingTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualBindingTierStore {
    pub items: RefCell<BTreeMap<String, ManualBindingTier>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualBindingTierStore for MemoryManualBindingTierStore {
    fn find(&self, code: &str) -> Option<ManualBindingTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualBindingTier) {
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

    fn list(&self) -> Vec<ManualBindingTier> {
        let mut out: Vec<ManualBindingTier> = Vec::new();
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

pub fn new_memory_manual_binding_tier_store() -> Rc<dyn ManualBindingTierStore> {
    Rc::new(MemoryManualBindingTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualBindingTierStore {
    pub inner: Rc<dyn ManualBindingTierStore>,
    pub cache: RefCell<cache::TtlCache<ManualBindingTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualBindingTierStore for CachedManualBindingTierStore {
    fn find(&self, code: &str) -> Option<ManualBindingTier> {
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

    fn save(&self, item: &ManualBindingTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualBindingTier> {
        self.inner.list()
    }
}

pub fn new_cached_manual_binding_tier_store(
    inner: Rc<dyn ManualBindingTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualBindingTierStore> {
    Rc::new(CachedManualBindingTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualBindingTier>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual binding tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBindingTierQuery {
    pub unit: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualBindingTierQuery {
    pub fn matches(&self, item: &ManualBindingTier) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualBindingTier]) -> Vec<ManualBindingTier> {
        let matched = collections::filter(items, |candidate: &ManualBindingTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_binding_tiers(
    store: Rc<dyn ManualBindingTierStore>,
    query: &ManualBindingTierQuery,
) -> Vec<ManualBindingTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualBindingTierStore> = new_memory_manual_binding_tier_store();
        let item = new_manual_binding_tier("code-981", "label-524");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
