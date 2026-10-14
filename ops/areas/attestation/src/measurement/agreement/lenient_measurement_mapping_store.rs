use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient measurement mappings by code.
pub trait LenientMeasurementMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientMeasurementMapping>;
    fn save(&self, item: &LenientMeasurementMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientMeasurementMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientMeasurementMappingStore {
    pub items: RefCell<BTreeMap<String, LenientMeasurementMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientMeasurementMappingStore for MemoryLenientMeasurementMappingStore {
    fn find(&self, code: &str) -> Option<LenientMeasurementMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientMeasurementMapping) {
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

    fn list(&self) -> Vec<LenientMeasurementMapping> {
        let mut out: Vec<LenientMeasurementMapping> = Vec::new();
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

pub fn new_memory_lenient_measurement_mapping_store() -> Rc<dyn LenientMeasurementMappingStore> {
    Rc::new(MemoryLenientMeasurementMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientMeasurementMappingStore {
    pub inner: Rc<dyn LenientMeasurementMappingStore>,
    pub cache: RefCell<cache::TtlCache<LenientMeasurementMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientMeasurementMappingStore for CachedLenientMeasurementMappingStore {
    fn find(&self, code: &str) -> Option<LenientMeasurementMapping> {
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

    fn save(&self, item: &LenientMeasurementMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientMeasurementMapping> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_measurement_mapping_store(
    inner: Rc<dyn LenientMeasurementMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientMeasurementMappingStore> {
    Rc::new(CachedLenientMeasurementMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientMeasurementMapping>(
            64,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient measurement mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementMappingQuery {
    pub channel: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientMeasurementMappingQuery {
    pub fn matches(&self, item: &LenientMeasurementMapping) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientMeasurementMapping]) -> Vec<LenientMeasurementMapping> {
        let matched = collections::filter(items, |candidate: &LenientMeasurementMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_measurement_mappings(
    store: Rc<dyn LenientMeasurementMappingStore>,
    query: &LenientMeasurementMappingQuery,
) -> Vec<LenientMeasurementMapping> {
    query.apply(&store.list())
}
