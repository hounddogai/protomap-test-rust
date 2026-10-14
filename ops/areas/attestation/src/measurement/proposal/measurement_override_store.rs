use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps measurement overrides by code.
pub trait MeasurementOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MeasurementOverride>;
    fn save(&self, item: &MeasurementOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MeasurementOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMeasurementOverrideStore {
    pub items: RefCell<BTreeMap<String, MeasurementOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl MeasurementOverrideStore for MemoryMeasurementOverrideStore {
    fn find(&self, code: &str) -> Option<MeasurementOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MeasurementOverride) {
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

    fn list(&self) -> Vec<MeasurementOverride> {
        let mut out: Vec<MeasurementOverride> = Vec::new();
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

pub fn new_memory_measurement_override_store() -> Rc<dyn MeasurementOverrideStore> {
    Rc::new(MemoryMeasurementOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMeasurementOverrideStore {
    pub inner: Rc<dyn MeasurementOverrideStore>,
    pub cache: RefCell<cache::TtlCache<MeasurementOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MeasurementOverrideStore for CachedMeasurementOverrideStore {
    fn find(&self, code: &str) -> Option<MeasurementOverride> {
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

    fn save(&self, item: &MeasurementOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MeasurementOverride> {
        self.inner.list()
    }
}

pub fn new_cached_measurement_override_store(
    inner: Rc<dyn MeasurementOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MeasurementOverrideStore> {
    Rc::new(CachedMeasurementOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MeasurementOverride>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects measurement overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementOverrideQuery {
    pub label: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MeasurementOverrideQuery {
    pub fn matches(&self, item: &MeasurementOverride) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MeasurementOverride]) -> Vec<MeasurementOverride> {
        let matched = collections::filter(items, |candidate: &MeasurementOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_measurement_overrides(
    store: Rc<dyn MeasurementOverrideStore>,
    query: &MeasurementOverrideQuery,
) -> Vec<MeasurementOverride> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn MeasurementOverrideStore> = new_memory_measurement_override_store();
        let item = new_measurement_override("code-591", "label-839");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
