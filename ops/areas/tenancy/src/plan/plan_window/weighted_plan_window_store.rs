use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted plan windows by code.
pub trait WeightedPlanWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedPlanWindow>;
    fn save(&self, item: &WeightedPlanWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedPlanWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedPlanWindowStore {
    pub items: RefCell<BTreeMap<String, WeightedPlanWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedPlanWindowStore for MemoryWeightedPlanWindowStore {
    fn find(&self, code: &str) -> Option<WeightedPlanWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedPlanWindow) {
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

    fn list(&self) -> Vec<WeightedPlanWindow> {
        let mut out: Vec<WeightedPlanWindow> = Vec::new();
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

pub fn new_memory_weighted_plan_window_store() -> Rc<dyn WeightedPlanWindowStore> {
    Rc::new(MemoryWeightedPlanWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedPlanWindowStore {
    pub inner: Rc<dyn WeightedPlanWindowStore>,
    pub cache: RefCell<cache::TtlCache<WeightedPlanWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedPlanWindowStore for CachedWeightedPlanWindowStore {
    fn find(&self, code: &str) -> Option<WeightedPlanWindow> {
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

    fn save(&self, item: &WeightedPlanWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedPlanWindow> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_plan_window_store(
    inner: Rc<dyn WeightedPlanWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedPlanWindowStore> {
    Rc::new(CachedWeightedPlanWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedPlanWindow>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted plan windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPlanWindowQuery {
    pub unit: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedPlanWindowQuery {
    pub fn matches(&self, item: &WeightedPlanWindow) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedPlanWindow]) -> Vec<WeightedPlanWindow> {
        let matched = collections::filter(items, |candidate: &WeightedPlanWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_plan_windows(
    store: Rc<dyn WeightedPlanWindowStore>,
    query: &WeightedPlanWindowQuery,
) -> Vec<WeightedPlanWindow> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedPlanWindowStore> = new_memory_weighted_plan_window_store();
        let item = new_weighted_plan_window("code-295", "title-336");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
