use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy rollout proposals by code.
pub trait LegacyRolloutProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyRolloutProposal>;
    fn save(&self, item: &LegacyRolloutProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyRolloutProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyRolloutProposalStore {
    pub items: RefCell<BTreeMap<String, LegacyRolloutProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyRolloutProposalStore for MemoryLegacyRolloutProposalStore {
    fn find(&self, code: &str) -> Option<LegacyRolloutProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyRolloutProposal) {
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

    fn list(&self) -> Vec<LegacyRolloutProposal> {
        let mut out: Vec<LegacyRolloutProposal> = Vec::new();
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

pub fn new_memory_legacy_rollout_proposal_store() -> Rc<dyn LegacyRolloutProposalStore> {
    Rc::new(MemoryLegacyRolloutProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyRolloutProposalStore {
    pub inner: Rc<dyn LegacyRolloutProposalStore>,
    pub cache: RefCell<cache::TtlCache<LegacyRolloutProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyRolloutProposalStore for CachedLegacyRolloutProposalStore {
    fn find(&self, code: &str) -> Option<LegacyRolloutProposal> {
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

    fn save(&self, item: &LegacyRolloutProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyRolloutProposal> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_rollout_proposal_store(
    inner: Rc<dyn LegacyRolloutProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyRolloutProposalStore> {
    Rc::new(CachedLegacyRolloutProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyRolloutProposal>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy rollout proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutProposalQuery {
    pub segment: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyRolloutProposalQuery {
    pub fn matches(&self, item: &LegacyRolloutProposal) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyRolloutProposal]) -> Vec<LegacyRolloutProposal> {
        let matched = collections::filter(items, |candidate: &LegacyRolloutProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_rollout_proposals(
    store: Rc<dyn LegacyRolloutProposalStore>,
    query: &LegacyRolloutProposalQuery,
) -> Vec<LegacyRolloutProposal> {
    query.apply(&store.list())
}
