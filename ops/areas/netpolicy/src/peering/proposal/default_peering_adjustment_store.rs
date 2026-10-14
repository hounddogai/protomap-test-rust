use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default peering adjustments by code.
pub trait DefaultPeeringAdjustmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultPeeringAdjustment>;
    fn save(&self, item: &DefaultPeeringAdjustment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultPeeringAdjustment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultPeeringAdjustmentStore {
    pub items: RefCell<BTreeMap<String, DefaultPeeringAdjustment>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultPeeringAdjustmentStore for MemoryDefaultPeeringAdjustmentStore {
    fn find(&self, code: &str) -> Option<DefaultPeeringAdjustment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultPeeringAdjustment) {
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

    fn list(&self) -> Vec<DefaultPeeringAdjustment> {
        let mut out: Vec<DefaultPeeringAdjustment> = Vec::new();
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

pub fn new_memory_default_peering_adjustment_store() -> Rc<dyn DefaultPeeringAdjustmentStore> {
    Rc::new(MemoryDefaultPeeringAdjustmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultPeeringAdjustmentStore {
    pub inner: Rc<dyn DefaultPeeringAdjustmentStore>,
    pub cache: RefCell<cache::TtlCache<DefaultPeeringAdjustment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultPeeringAdjustmentStore for CachedDefaultPeeringAdjustmentStore {
    fn find(&self, code: &str) -> Option<DefaultPeeringAdjustment> {
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

    fn save(&self, item: &DefaultPeeringAdjustment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultPeeringAdjustment> {
        self.inner.list()
    }
}

pub fn new_cached_default_peering_adjustment_store(
    inner: Rc<dyn DefaultPeeringAdjustmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultPeeringAdjustmentStore> {
    Rc::new(CachedDefaultPeeringAdjustmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultPeeringAdjustment>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default peering adjustments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringAdjustmentQuery {
    pub region: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultPeeringAdjustmentQuery {
    pub fn matches(&self, item: &DefaultPeeringAdjustment) -> bool {
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

    pub fn apply(&self, items: &[DefaultPeeringAdjustment]) -> Vec<DefaultPeeringAdjustment> {
        let matched = collections::filter(items, |candidate: &DefaultPeeringAdjustment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_peering_adjustments(
    store: Rc<dyn DefaultPeeringAdjustmentStore>,
    query: &DefaultPeeringAdjustmentQuery,
) -> Vec<DefaultPeeringAdjustment> {
    query.apply(&store.list())
}
