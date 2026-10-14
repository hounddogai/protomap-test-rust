use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps isolation proposals by code.
pub trait IsolationProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<IsolationProposal>;
    fn save(&self, item: &IsolationProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<IsolationProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryIsolationProposalStore {
    pub items: RefCell<BTreeMap<String, IsolationProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl IsolationProposalStore for MemoryIsolationProposalStore {
    fn find(&self, code: &str) -> Option<IsolationProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &IsolationProposal) {
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

    fn list(&self) -> Vec<IsolationProposal> {
        let mut out: Vec<IsolationProposal> = Vec::new();
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

pub fn new_memory_isolation_proposal_store() -> Rc<dyn IsolationProposalStore> {
    Rc::new(MemoryIsolationProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedIsolationProposalStore {
    pub inner: Rc<dyn IsolationProposalStore>,
    pub cache: RefCell<cache::TtlCache<IsolationProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl IsolationProposalStore for CachedIsolationProposalStore {
    fn find(&self, code: &str) -> Option<IsolationProposal> {
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

    fn save(&self, item: &IsolationProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<IsolationProposal> {
        self.inner.list()
    }
}

pub fn new_cached_isolation_proposal_store(
    inner: Rc<dyn IsolationProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn IsolationProposalStore> {
    Rc::new(CachedIsolationProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<IsolationProposal>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects isolation proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationProposalQuery {
    pub source: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl IsolationProposalQuery {
    pub fn matches(&self, item: &IsolationProposal) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
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

    pub fn apply(&self, items: &[IsolationProposal]) -> Vec<IsolationProposal> {
        let matched = collections::filter(items, |candidate: &IsolationProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_isolation_proposals(
    store: Rc<dyn IsolationProposalStore>,
    query: &IsolationProposalQuery,
) -> Vec<IsolationProposal> {
    query.apply(&store.list())
}
