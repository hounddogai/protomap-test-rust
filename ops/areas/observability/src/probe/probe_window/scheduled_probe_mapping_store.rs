use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled probe mappings by code.
pub trait ScheduledProbeMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledProbeMapping>;
    fn save(&self, item: &ScheduledProbeMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledProbeMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledProbeMappingStore {
    pub items: RefCell<BTreeMap<String, ScheduledProbeMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledProbeMappingStore for MemoryScheduledProbeMappingStore {
    fn find(&self, code: &str) -> Option<ScheduledProbeMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledProbeMapping) {
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

    fn list(&self) -> Vec<ScheduledProbeMapping> {
        let mut out: Vec<ScheduledProbeMapping> = Vec::new();
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

pub fn new_memory_scheduled_probe_mapping_store() -> Rc<dyn ScheduledProbeMappingStore> {
    Rc::new(MemoryScheduledProbeMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledProbeMappingStore {
    pub inner: Rc<dyn ScheduledProbeMappingStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledProbeMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledProbeMappingStore for CachedScheduledProbeMappingStore {
    fn find(&self, code: &str) -> Option<ScheduledProbeMapping> {
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

    fn save(&self, item: &ScheduledProbeMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledProbeMapping> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_probe_mapping_store(
    inner: Rc<dyn ScheduledProbeMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledProbeMappingStore> {
    Rc::new(CachedScheduledProbeMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledProbeMapping>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled probe mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeMappingQuery {
    pub region: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledProbeMappingQuery {
    pub fn matches(&self, item: &ScheduledProbeMapping) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledProbeMapping]) -> Vec<ScheduledProbeMapping> {
        let matched = collections::filter(items, |candidate: &ScheduledProbeMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_probe_mappings(
    store: Rc<dyn ScheduledProbeMappingStore>,
    query: &ScheduledProbeMappingQuery,
) -> Vec<ScheduledProbeMapping> {
    query.apply(&store.list())
}
