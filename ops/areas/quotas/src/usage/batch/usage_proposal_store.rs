use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps usage proposals by code.
pub trait UsageProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<UsageProposal>;
    fn save(&self, item: &UsageProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<UsageProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryUsageProposalStore {
    pub items: RefCell<BTreeMap<String, UsageProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl UsageProposalStore for MemoryUsageProposalStore {
    fn find(&self, code: &str) -> Option<UsageProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &UsageProposal) {
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

    fn list(&self) -> Vec<UsageProposal> {
        let mut out: Vec<UsageProposal> = Vec::new();
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

pub fn new_memory_usage_proposal_store() -> Rc<dyn UsageProposalStore> {
    Rc::new(MemoryUsageProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedUsageProposalStore {
    pub inner: Rc<dyn UsageProposalStore>,
    pub cache: RefCell<cache::TtlCache<UsageProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl UsageProposalStore for CachedUsageProposalStore {
    fn find(&self, code: &str) -> Option<UsageProposal> {
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

    fn save(&self, item: &UsageProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<UsageProposal> {
        self.inner.list()
    }
}

pub fn new_cached_usage_proposal_store(
    inner: Rc<dyn UsageProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn UsageProposalStore> {
    Rc::new(CachedUsageProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<UsageProposal>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects usage proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageProposalQuery {
    pub scope: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl UsageProposalQuery {
    pub fn matches(&self, item: &UsageProposal) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
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

    pub fn apply(&self, items: &[UsageProposal]) -> Vec<UsageProposal> {
        let matched = collections::filter(items, |candidate: &UsageProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_usage_proposals(store: Rc<dyn UsageProposalStore>, query: &UsageProposalQuery) -> Vec<UsageProposal> {
    query.apply(&store.list())
}
