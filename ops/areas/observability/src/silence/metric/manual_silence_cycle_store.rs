use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual silence cycles by code.
pub trait ManualSilenceCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualSilenceCycle>;
    fn save(&self, item: &ManualSilenceCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualSilenceCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualSilenceCycleStore {
    pub items: RefCell<BTreeMap<String, ManualSilenceCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualSilenceCycleStore for MemoryManualSilenceCycleStore {
    fn find(&self, code: &str) -> Option<ManualSilenceCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualSilenceCycle) {
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

    fn list(&self) -> Vec<ManualSilenceCycle> {
        let mut out: Vec<ManualSilenceCycle> = Vec::new();
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

pub fn new_memory_manual_silence_cycle_store() -> Rc<dyn ManualSilenceCycleStore> {
    Rc::new(MemoryManualSilenceCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualSilenceCycleStore {
    pub inner: Rc<dyn ManualSilenceCycleStore>,
    pub cache: RefCell<cache::TtlCache<ManualSilenceCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualSilenceCycleStore for CachedManualSilenceCycleStore {
    fn find(&self, code: &str) -> Option<ManualSilenceCycle> {
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

    fn save(&self, item: &ManualSilenceCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualSilenceCycle> {
        self.inner.list()
    }
}

pub fn new_cached_manual_silence_cycle_store(
    inner: Rc<dyn ManualSilenceCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualSilenceCycleStore> {
    Rc::new(CachedManualSilenceCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualSilenceCycle>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual silence cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceCycleQuery {
    pub name: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualSilenceCycleQuery {
    pub fn matches(&self, item: &ManualSilenceCycle) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[ManualSilenceCycle]) -> Vec<ManualSilenceCycle> {
        let matched = collections::filter(items, |candidate: &ManualSilenceCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_silence_cycles(
    store: Rc<dyn ManualSilenceCycleStore>,
    query: &ManualSilenceCycleQuery,
) -> Vec<ManualSilenceCycle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualSilenceCycleStore> = new_memory_manual_silence_cycle_store();
        let item = new_manual_silence_cycle("code-233", "name-706");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
