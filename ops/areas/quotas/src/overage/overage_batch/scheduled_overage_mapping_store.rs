use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled overage mappings by code.
pub trait ScheduledOverageMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledOverageMapping>;
    fn save(&self, item: &ScheduledOverageMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledOverageMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledOverageMappingStore {
    pub items: RefCell<BTreeMap<String, ScheduledOverageMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledOverageMappingStore for MemoryScheduledOverageMappingStore {
    fn find(&self, code: &str) -> Option<ScheduledOverageMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledOverageMapping) {
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

    fn list(&self) -> Vec<ScheduledOverageMapping> {
        let mut out: Vec<ScheduledOverageMapping> = Vec::new();
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

pub fn new_memory_scheduled_overage_mapping_store() -> Rc<dyn ScheduledOverageMappingStore> {
    Rc::new(MemoryScheduledOverageMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledOverageMappingStore {
    pub inner: Rc<dyn ScheduledOverageMappingStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledOverageMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledOverageMappingStore for CachedScheduledOverageMappingStore {
    fn find(&self, code: &str) -> Option<ScheduledOverageMapping> {
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

    fn save(&self, item: &ScheduledOverageMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledOverageMapping> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_overage_mapping_store(
    inner: Rc<dyn ScheduledOverageMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledOverageMappingStore> {
    Rc::new(CachedScheduledOverageMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledOverageMapping>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled overage mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledOverageMappingQuery {
    pub title: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledOverageMappingQuery {
    pub fn matches(&self, item: &ScheduledOverageMapping) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledOverageMapping]) -> Vec<ScheduledOverageMapping> {
        let matched = collections::filter(items, |candidate: &ScheduledOverageMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_overage_mappings(
    store: Rc<dyn ScheduledOverageMappingStore>,
    query: &ScheduledOverageMappingQuery,
) -> Vec<ScheduledOverageMapping> {
    query.apply(&store.list())
}
