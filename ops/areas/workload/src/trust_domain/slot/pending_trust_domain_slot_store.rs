use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending trust domain slots by code.
pub trait PendingTrustDomainSlotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingTrustDomainSlot>;
    fn save(&self, item: &PendingTrustDomainSlot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingTrustDomainSlot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingTrustDomainSlotStore {
    pub items: RefCell<BTreeMap<String, PendingTrustDomainSlot>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingTrustDomainSlotStore for MemoryPendingTrustDomainSlotStore {
    fn find(&self, code: &str) -> Option<PendingTrustDomainSlot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingTrustDomainSlot) {
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

    fn list(&self) -> Vec<PendingTrustDomainSlot> {
        let mut out: Vec<PendingTrustDomainSlot> = Vec::new();
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

pub fn new_memory_pending_trust_domain_slot_store() -> Rc<dyn PendingTrustDomainSlotStore> {
    Rc::new(MemoryPendingTrustDomainSlotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingTrustDomainSlotStore {
    pub inner: Rc<dyn PendingTrustDomainSlotStore>,
    pub cache: RefCell<cache::TtlCache<PendingTrustDomainSlot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingTrustDomainSlotStore for CachedPendingTrustDomainSlotStore {
    fn find(&self, code: &str) -> Option<PendingTrustDomainSlot> {
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

    fn save(&self, item: &PendingTrustDomainSlot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingTrustDomainSlot> {
        self.inner.list()
    }
}

pub fn new_cached_pending_trust_domain_slot_store(
    inner: Rc<dyn PendingTrustDomainSlotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingTrustDomainSlotStore> {
    Rc::new(CachedPendingTrustDomainSlotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingTrustDomainSlot>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending trust domain slots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTrustDomainSlotQuery {
    pub source: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingTrustDomainSlotQuery {
    pub fn matches(&self, item: &PendingTrustDomainSlot) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
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

    pub fn apply(&self, items: &[PendingTrustDomainSlot]) -> Vec<PendingTrustDomainSlot> {
        let matched = collections::filter(items, |candidate: &PendingTrustDomainSlot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_trust_domain_slots(
    store: Rc<dyn PendingTrustDomainSlotStore>,
    query: &PendingTrustDomainSlotQuery,
) -> Vec<PendingTrustDomainSlot> {
    query.apply(&store.list())
}
