use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps peering claims by code.
pub trait PeeringClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PeeringClaim>;
    fn save(&self, item: &PeeringClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PeeringClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPeeringClaimStore {
    pub items: RefCell<BTreeMap<String, PeeringClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl PeeringClaimStore for MemoryPeeringClaimStore {
    fn find(&self, code: &str) -> Option<PeeringClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PeeringClaim) {
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

    fn list(&self) -> Vec<PeeringClaim> {
        let mut out: Vec<PeeringClaim> = Vec::new();
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

pub fn new_memory_peering_claim_store() -> Rc<dyn PeeringClaimStore> {
    Rc::new(MemoryPeeringClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPeeringClaimStore {
    pub inner: Rc<dyn PeeringClaimStore>,
    pub cache: RefCell<cache::TtlCache<PeeringClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PeeringClaimStore for CachedPeeringClaimStore {
    fn find(&self, code: &str) -> Option<PeeringClaim> {
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

    fn save(&self, item: &PeeringClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PeeringClaim> {
        self.inner.list()
    }
}

pub fn new_cached_peering_claim_store(
    inner: Rc<dyn PeeringClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PeeringClaimStore> {
    Rc::new(CachedPeeringClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PeeringClaim>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects peering claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringClaimQuery {
    pub label: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PeeringClaimQuery {
    pub fn matches(&self, item: &PeeringClaim) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PeeringClaim]) -> Vec<PeeringClaim> {
        let matched = collections::filter(items, |candidate: &PeeringClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_peering_claims(store: Rc<dyn PeeringClaimStore>, query: &PeeringClaimQuery) -> Vec<PeeringClaim> {
    query.apply(&store.list())
}
