use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered rollout proposals by code.
pub trait TieredRolloutProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredRolloutProposal>;
    fn save(&self, item: &TieredRolloutProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredRolloutProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredRolloutProposalStore {
    pub items: RefCell<BTreeMap<String, TieredRolloutProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredRolloutProposalStore for MemoryTieredRolloutProposalStore {
    fn find(&self, code: &str) -> Option<TieredRolloutProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredRolloutProposal) {
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

    fn list(&self) -> Vec<TieredRolloutProposal> {
        let mut out: Vec<TieredRolloutProposal> = Vec::new();
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

pub fn new_memory_tiered_rollout_proposal_store() -> Rc<dyn TieredRolloutProposalStore> {
    Rc::new(MemoryTieredRolloutProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredRolloutProposalStore {
    pub inner: Rc<dyn TieredRolloutProposalStore>,
    pub cache: RefCell<cache::TtlCache<TieredRolloutProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredRolloutProposalStore for CachedTieredRolloutProposalStore {
    fn find(&self, code: &str) -> Option<TieredRolloutProposal> {
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

    fn save(&self, item: &TieredRolloutProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredRolloutProposal> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_rollout_proposal_store(
    inner: Rc<dyn TieredRolloutProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredRolloutProposalStore> {
    Rc::new(CachedTieredRolloutProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredRolloutProposal>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered rollout proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutProposalQuery {
    pub label: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredRolloutProposalQuery {
    pub fn matches(&self, item: &TieredRolloutProposal) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredRolloutProposal]) -> Vec<TieredRolloutProposal> {
        let matched = collections::filter(items, |candidate: &TieredRolloutProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_rollout_proposals(
    store: Rc<dyn TieredRolloutProposalStore>,
    query: &TieredRolloutProposalQuery,
) -> Vec<TieredRolloutProposal> {
    query.apply(&store.list())
}
