use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps approval proposals by code.
pub trait ApprovalProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ApprovalProposal>;
    fn save(&self, item: &ApprovalProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ApprovalProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryApprovalProposalStore {
    pub items: RefCell<BTreeMap<String, ApprovalProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl ApprovalProposalStore for MemoryApprovalProposalStore {
    fn find(&self, code: &str) -> Option<ApprovalProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ApprovalProposal) {
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

    fn list(&self) -> Vec<ApprovalProposal> {
        let mut out: Vec<ApprovalProposal> = Vec::new();
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

pub fn new_memory_approval_proposal_store() -> Rc<dyn ApprovalProposalStore> {
    Rc::new(MemoryApprovalProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedApprovalProposalStore {
    pub inner: Rc<dyn ApprovalProposalStore>,
    pub cache: RefCell<cache::TtlCache<ApprovalProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ApprovalProposalStore for CachedApprovalProposalStore {
    fn find(&self, code: &str) -> Option<ApprovalProposal> {
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

    fn save(&self, item: &ApprovalProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ApprovalProposal> {
        self.inner.list()
    }
}

pub fn new_cached_approval_proposal_store(
    inner: Rc<dyn ApprovalProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ApprovalProposalStore> {
    Rc::new(CachedApprovalProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ApprovalProposal>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects approval proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalProposalQuery {
    pub unit: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ApprovalProposalQuery {
    pub fn matches(&self, item: &ApprovalProposal) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ApprovalProposal]) -> Vec<ApprovalProposal> {
        let matched = collections::filter(items, |candidate: &ApprovalProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_approval_proposals(
    store: Rc<dyn ApprovalProposalStore>,
    query: &ApprovalProposalQuery,
) -> Vec<ApprovalProposal> {
    query.apply(&store.list())
}
