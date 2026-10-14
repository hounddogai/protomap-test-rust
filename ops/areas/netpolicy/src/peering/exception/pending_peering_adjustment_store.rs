use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending peering adjustments by code.
pub trait PendingPeeringAdjustmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingPeeringAdjustment>;
    fn save(&self, item: &PendingPeeringAdjustment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingPeeringAdjustment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingPeeringAdjustmentStore {
    pub items: RefCell<BTreeMap<String, PendingPeeringAdjustment>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingPeeringAdjustmentStore for MemoryPendingPeeringAdjustmentStore {
    fn find(&self, code: &str) -> Option<PendingPeeringAdjustment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingPeeringAdjustment) {
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

    fn list(&self) -> Vec<PendingPeeringAdjustment> {
        let mut out: Vec<PendingPeeringAdjustment> = Vec::new();
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

pub fn new_memory_pending_peering_adjustment_store() -> Rc<dyn PendingPeeringAdjustmentStore> {
    Rc::new(MemoryPendingPeeringAdjustmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingPeeringAdjustmentStore {
    pub inner: Rc<dyn PendingPeeringAdjustmentStore>,
    pub cache: RefCell<cache::TtlCache<PendingPeeringAdjustment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingPeeringAdjustmentStore for CachedPendingPeeringAdjustmentStore {
    fn find(&self, code: &str) -> Option<PendingPeeringAdjustment> {
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

    fn save(&self, item: &PendingPeeringAdjustment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingPeeringAdjustment> {
        self.inner.list()
    }
}

pub fn new_cached_pending_peering_adjustment_store(
    inner: Rc<dyn PendingPeeringAdjustmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingPeeringAdjustmentStore> {
    Rc::new(CachedPendingPeeringAdjustmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingPeeringAdjustment>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending peering adjustments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPeeringAdjustmentQuery {
    pub cluster: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingPeeringAdjustmentQuery {
    pub fn matches(&self, item: &PendingPeeringAdjustment) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[PendingPeeringAdjustment]) -> Vec<PendingPeeringAdjustment> {
        let matched = collections::filter(items, |candidate: &PendingPeeringAdjustment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_peering_adjustments(
    store: Rc<dyn PendingPeeringAdjustmentStore>,
    query: &PendingPeeringAdjustmentQuery,
) -> Vec<PendingPeeringAdjustment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PendingPeeringAdjustmentStore> = new_memory_pending_peering_adjustment_store();
        let item = new_pending_peering_adjustment("code-501", "name-576");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
