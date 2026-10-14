use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps reservation proposals by code.
pub trait ReservationProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ReservationProposal>;
    fn save(&self, item: &ReservationProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ReservationProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryReservationProposalStore {
    pub items: RefCell<BTreeMap<String, ReservationProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl ReservationProposalStore for MemoryReservationProposalStore {
    fn find(&self, code: &str) -> Option<ReservationProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ReservationProposal) {
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

    fn list(&self) -> Vec<ReservationProposal> {
        let mut out: Vec<ReservationProposal> = Vec::new();
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

pub fn new_memory_reservation_proposal_store() -> Rc<dyn ReservationProposalStore> {
    Rc::new(MemoryReservationProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedReservationProposalStore {
    pub inner: Rc<dyn ReservationProposalStore>,
    pub cache: RefCell<cache::TtlCache<ReservationProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ReservationProposalStore for CachedReservationProposalStore {
    fn find(&self, code: &str) -> Option<ReservationProposal> {
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

    fn save(&self, item: &ReservationProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ReservationProposal> {
        self.inner.list()
    }
}

pub fn new_cached_reservation_proposal_store(
    inner: Rc<dyn ReservationProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ReservationProposalStore> {
    Rc::new(CachedReservationProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ReservationProposal>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects reservation proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationProposalQuery {
    pub region: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ReservationProposalQuery {
    pub fn matches(&self, item: &ReservationProposal) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ReservationProposal]) -> Vec<ReservationProposal> {
        let matched = collections::filter(items, |candidate: &ReservationProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_reservation_proposals(
    store: Rc<dyn ReservationProposalStore>,
    query: &ReservationProposalQuery,
) -> Vec<ReservationProposal> {
    query.apply(&store.list())
}
