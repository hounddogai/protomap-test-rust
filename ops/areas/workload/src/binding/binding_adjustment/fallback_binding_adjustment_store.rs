use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback binding adjustments by code.
pub trait FallbackBindingAdjustmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackBindingAdjustment>;
    fn save(&self, item: &FallbackBindingAdjustment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackBindingAdjustment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackBindingAdjustmentStore {
    pub items: RefCell<BTreeMap<String, FallbackBindingAdjustment>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackBindingAdjustmentStore for MemoryFallbackBindingAdjustmentStore {
    fn find(&self, code: &str) -> Option<FallbackBindingAdjustment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackBindingAdjustment) {
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

    fn list(&self) -> Vec<FallbackBindingAdjustment> {
        let mut out: Vec<FallbackBindingAdjustment> = Vec::new();
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

pub fn new_memory_fallback_binding_adjustment_store() -> Rc<dyn FallbackBindingAdjustmentStore> {
    Rc::new(MemoryFallbackBindingAdjustmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackBindingAdjustmentStore {
    pub inner: Rc<dyn FallbackBindingAdjustmentStore>,
    pub cache: RefCell<cache::TtlCache<FallbackBindingAdjustment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackBindingAdjustmentStore for CachedFallbackBindingAdjustmentStore {
    fn find(&self, code: &str) -> Option<FallbackBindingAdjustment> {
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

    fn save(&self, item: &FallbackBindingAdjustment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackBindingAdjustment> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_binding_adjustment_store(
    inner: Rc<dyn FallbackBindingAdjustmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackBindingAdjustmentStore> {
    Rc::new(CachedFallbackBindingAdjustmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackBindingAdjustment>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback binding adjustments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingAdjustmentQuery {
    pub name: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackBindingAdjustmentQuery {
    pub fn matches(&self, item: &FallbackBindingAdjustment) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackBindingAdjustment]) -> Vec<FallbackBindingAdjustment> {
        let matched = collections::filter(items, |candidate: &FallbackBindingAdjustment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_binding_adjustments(
    store: Rc<dyn FallbackBindingAdjustmentStore>,
    query: &FallbackBindingAdjustmentQuery,
) -> Vec<FallbackBindingAdjustment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackBindingAdjustmentStore> = new_memory_fallback_binding_adjustment_store();
        let item = new_fallback_binding_adjustment("code-825", "name-51");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
