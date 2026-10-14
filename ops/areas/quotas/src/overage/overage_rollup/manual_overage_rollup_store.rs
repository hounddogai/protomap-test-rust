use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual overage rollups by code.
pub trait ManualOverageRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualOverageRollup>;
    fn save(&self, item: &ManualOverageRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualOverageRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualOverageRollupStore {
    pub items: RefCell<BTreeMap<String, ManualOverageRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualOverageRollupStore for MemoryManualOverageRollupStore {
    fn find(&self, code: &str) -> Option<ManualOverageRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualOverageRollup) {
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

    fn list(&self) -> Vec<ManualOverageRollup> {
        let mut out: Vec<ManualOverageRollup> = Vec::new();
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

pub fn new_memory_manual_overage_rollup_store() -> Rc<dyn ManualOverageRollupStore> {
    Rc::new(MemoryManualOverageRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualOverageRollupStore {
    pub inner: Rc<dyn ManualOverageRollupStore>,
    pub cache: RefCell<cache::TtlCache<ManualOverageRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualOverageRollupStore for CachedManualOverageRollupStore {
    fn find(&self, code: &str) -> Option<ManualOverageRollup> {
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

    fn save(&self, item: &ManualOverageRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualOverageRollup> {
        self.inner.list()
    }
}

pub fn new_cached_manual_overage_rollup_store(
    inner: Rc<dyn ManualOverageRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualOverageRollupStore> {
    Rc::new(CachedManualOverageRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualOverageRollup>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual overage rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualOverageRollupQuery {
    pub segment: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualOverageRollupQuery {
    pub fn matches(&self, item: &ManualOverageRollup) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualOverageRollup]) -> Vec<ManualOverageRollup> {
        let matched = collections::filter(items, |candidate: &ManualOverageRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_overage_rollups(
    store: Rc<dyn ManualOverageRollupStore>,
    query: &ManualOverageRollupQuery,
) -> Vec<ManualOverageRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualOverageRollupStore> = new_memory_manual_overage_rollup_store();
        let item = new_manual_overage_rollup("code-611", "title-236");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
