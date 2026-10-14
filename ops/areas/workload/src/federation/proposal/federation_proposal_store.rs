use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps federation proposals by code.
pub trait FederationProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FederationProposal>;
    fn save(&self, item: &FederationProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FederationProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFederationProposalStore {
    pub items: RefCell<BTreeMap<String, FederationProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl FederationProposalStore for MemoryFederationProposalStore {
    fn find(&self, code: &str) -> Option<FederationProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FederationProposal) {
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

    fn list(&self) -> Vec<FederationProposal> {
        let mut out: Vec<FederationProposal> = Vec::new();
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

pub fn new_memory_federation_proposal_store() -> Rc<dyn FederationProposalStore> {
    Rc::new(MemoryFederationProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFederationProposalStore {
    pub inner: Rc<dyn FederationProposalStore>,
    pub cache: RefCell<cache::TtlCache<FederationProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FederationProposalStore for CachedFederationProposalStore {
    fn find(&self, code: &str) -> Option<FederationProposal> {
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

    fn save(&self, item: &FederationProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FederationProposal> {
        self.inner.list()
    }
}

pub fn new_cached_federation_proposal_store(
    inner: Rc<dyn FederationProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FederationProposalStore> {
    Rc::new(CachedFederationProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FederationProposal>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects federation proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationProposalQuery {
    pub title: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FederationProposalQuery {
    pub fn matches(&self, item: &FederationProposal) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.expires_at < self.min_expires_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FederationProposal]) -> Vec<FederationProposal> {
        let matched = collections::filter(items, |candidate: &FederationProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_federation_proposals(
    store: Rc<dyn FederationProposalStore>,
    query: &FederationProposalQuery,
) -> Vec<FederationProposal> {
    query.apply(&store.list())
}
