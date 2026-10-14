use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual identity proposals by code.
pub trait ManualIdentityProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualIdentityProposal>;
    fn save(&self, item: &ManualIdentityProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualIdentityProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualIdentityProposalStore {
    pub items: RefCell<BTreeMap<String, ManualIdentityProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualIdentityProposalStore for MemoryManualIdentityProposalStore {
    fn find(&self, code: &str) -> Option<ManualIdentityProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualIdentityProposal) {
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

    fn list(&self) -> Vec<ManualIdentityProposal> {
        let mut out: Vec<ManualIdentityProposal> = Vec::new();
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

pub fn new_memory_manual_identity_proposal_store() -> Rc<dyn ManualIdentityProposalStore> {
    Rc::new(MemoryManualIdentityProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualIdentityProposalStore {
    pub inner: Rc<dyn ManualIdentityProposalStore>,
    pub cache: RefCell<cache::TtlCache<ManualIdentityProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualIdentityProposalStore for CachedManualIdentityProposalStore {
    fn find(&self, code: &str) -> Option<ManualIdentityProposal> {
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

    fn save(&self, item: &ManualIdentityProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualIdentityProposal> {
        self.inner.list()
    }
}

pub fn new_cached_manual_identity_proposal_store(
    inner: Rc<dyn ManualIdentityProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualIdentityProposalStore> {
    Rc::new(CachedManualIdentityProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualIdentityProposal>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual identity proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityProposalQuery {
    pub cluster: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualIdentityProposalQuery {
    pub fn matches(&self, item: &ManualIdentityProposal) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualIdentityProposal]) -> Vec<ManualIdentityProposal> {
        let matched = collections::filter(items, |candidate: &ManualIdentityProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_identity_proposals(
    store: Rc<dyn ManualIdentityProposalStore>,
    query: &ManualIdentityProposalQuery,
) -> Vec<ManualIdentityProposal> {
    query.apply(&store.list())
}
