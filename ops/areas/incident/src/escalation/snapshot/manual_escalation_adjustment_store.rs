use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual escalation adjustments by code.
pub trait ManualEscalationAdjustmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualEscalationAdjustment>;
    fn save(&self, item: &ManualEscalationAdjustment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualEscalationAdjustment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualEscalationAdjustmentStore {
    pub items: RefCell<BTreeMap<String, ManualEscalationAdjustment>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualEscalationAdjustmentStore for MemoryManualEscalationAdjustmentStore {
    fn find(&self, code: &str) -> Option<ManualEscalationAdjustment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualEscalationAdjustment) {
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

    fn list(&self) -> Vec<ManualEscalationAdjustment> {
        let mut out: Vec<ManualEscalationAdjustment> = Vec::new();
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

pub fn new_memory_manual_escalation_adjustment_store() -> Rc<dyn ManualEscalationAdjustmentStore> {
    Rc::new(MemoryManualEscalationAdjustmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualEscalationAdjustmentStore {
    pub inner: Rc<dyn ManualEscalationAdjustmentStore>,
    pub cache: RefCell<cache::TtlCache<ManualEscalationAdjustment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualEscalationAdjustmentStore for CachedManualEscalationAdjustmentStore {
    fn find(&self, code: &str) -> Option<ManualEscalationAdjustment> {
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

    fn save(&self, item: &ManualEscalationAdjustment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualEscalationAdjustment> {
        self.inner.list()
    }
}

pub fn new_cached_manual_escalation_adjustment_store(
    inner: Rc<dyn ManualEscalationAdjustmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualEscalationAdjustmentStore> {
    Rc::new(CachedManualEscalationAdjustmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualEscalationAdjustment>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual escalation adjustments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualEscalationAdjustmentQuery {
    pub channel: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualEscalationAdjustmentQuery {
    pub fn matches(&self, item: &ManualEscalationAdjustment) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualEscalationAdjustment]) -> Vec<ManualEscalationAdjustment> {
        let matched = collections::filter(items, |candidate: &ManualEscalationAdjustment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_escalation_adjustments(
    store: Rc<dyn ManualEscalationAdjustmentStore>,
    query: &ManualEscalationAdjustmentQuery,
) -> Vec<ManualEscalationAdjustment> {
    query.apply(&store.list())
}
