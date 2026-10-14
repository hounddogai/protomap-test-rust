use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps plan proposals by code.
pub trait PlanProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PlanProposal>;
    fn save(&self, item: &PlanProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PlanProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPlanProposalStore {
    pub items: RefCell<BTreeMap<String, PlanProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl PlanProposalStore for MemoryPlanProposalStore {
    fn find(&self, code: &str) -> Option<PlanProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PlanProposal) {
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

    fn list(&self) -> Vec<PlanProposal> {
        let mut out: Vec<PlanProposal> = Vec::new();
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

pub fn new_memory_plan_proposal_store() -> Rc<dyn PlanProposalStore> {
    Rc::new(MemoryPlanProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPlanProposalStore {
    pub inner: Rc<dyn PlanProposalStore>,
    pub cache: RefCell<cache::TtlCache<PlanProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PlanProposalStore for CachedPlanProposalStore {
    fn find(&self, code: &str) -> Option<PlanProposal> {
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

    fn save(&self, item: &PlanProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PlanProposal> {
        self.inner.list()
    }
}

pub fn new_cached_plan_proposal_store(
    inner: Rc<dyn PlanProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PlanProposalStore> {
    Rc::new(CachedPlanProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PlanProposal>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects plan proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanProposalQuery {
    pub unit: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PlanProposalQuery {
    pub fn matches(&self, item: &PlanProposal) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PlanProposal]) -> Vec<PlanProposal> {
        let matched = collections::filter(items, |candidate: &PlanProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_plan_proposals(store: Rc<dyn PlanProposalStore>, query: &PlanProposalQuery) -> Vec<PlanProposal> {
    query.apply(&store.list())
}
