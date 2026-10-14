use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps measurement rollups by code.
pub trait MeasurementRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MeasurementRollup>;
    fn save(&self, item: &MeasurementRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MeasurementRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMeasurementRollupStore {
    pub items: RefCell<BTreeMap<String, MeasurementRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl MeasurementRollupStore for MemoryMeasurementRollupStore {
    fn find(&self, code: &str) -> Option<MeasurementRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MeasurementRollup) {
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

    fn list(&self) -> Vec<MeasurementRollup> {
        let mut out: Vec<MeasurementRollup> = Vec::new();
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

pub fn new_memory_measurement_rollup_store() -> Rc<dyn MeasurementRollupStore> {
    Rc::new(MemoryMeasurementRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMeasurementRollupStore {
    pub inner: Rc<dyn MeasurementRollupStore>,
    pub cache: RefCell<cache::TtlCache<MeasurementRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MeasurementRollupStore for CachedMeasurementRollupStore {
    fn find(&self, code: &str) -> Option<MeasurementRollup> {
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

    fn save(&self, item: &MeasurementRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MeasurementRollup> {
        self.inner.list()
    }
}

pub fn new_cached_measurement_rollup_store(
    inner: Rc<dyn MeasurementRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MeasurementRollupStore> {
    Rc::new(CachedMeasurementRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MeasurementRollup>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects measurement rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRollupQuery {
    pub title: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MeasurementRollupQuery {
    pub fn matches(&self, item: &MeasurementRollup) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MeasurementRollup]) -> Vec<MeasurementRollup> {
        let matched = collections::filter(items, |candidate: &MeasurementRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_measurement_rollups(
    store: Rc<dyn MeasurementRollupStore>,
    query: &MeasurementRollupQuery,
) -> Vec<MeasurementRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn MeasurementRollupStore> = new_memory_measurement_rollup_store();
        let item = new_measurement_rollup("code-107", "title-844");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
