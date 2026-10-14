use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy flow log proposals by code.
pub trait LegacyFlowLogProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyFlowLogProposal>;
    fn save(&self, item: &LegacyFlowLogProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyFlowLogProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyFlowLogProposalStore {
    pub items: RefCell<BTreeMap<String, LegacyFlowLogProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyFlowLogProposalStore for MemoryLegacyFlowLogProposalStore {
    fn find(&self, code: &str) -> Option<LegacyFlowLogProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyFlowLogProposal) {
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

    fn list(&self) -> Vec<LegacyFlowLogProposal> {
        let mut out: Vec<LegacyFlowLogProposal> = Vec::new();
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

pub fn new_memory_legacy_flow_log_proposal_store() -> Rc<dyn LegacyFlowLogProposalStore> {
    Rc::new(MemoryLegacyFlowLogProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyFlowLogProposalStore {
    pub inner: Rc<dyn LegacyFlowLogProposalStore>,
    pub cache: RefCell<cache::TtlCache<LegacyFlowLogProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyFlowLogProposalStore for CachedLegacyFlowLogProposalStore {
    fn find(&self, code: &str) -> Option<LegacyFlowLogProposal> {
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

    fn save(&self, item: &LegacyFlowLogProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyFlowLogProposal> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_flow_log_proposal_store(
    inner: Rc<dyn LegacyFlowLogProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyFlowLogProposalStore> {
    Rc::new(CachedLegacyFlowLogProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyFlowLogProposal>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy flow log proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogProposalQuery {
    pub name: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyFlowLogProposalQuery {
    pub fn matches(&self, item: &LegacyFlowLogProposal) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyFlowLogProposal]) -> Vec<LegacyFlowLogProposal> {
        let matched = collections::filter(items, |candidate: &LegacyFlowLogProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_flow_log_proposals(
    store: Rc<dyn LegacyFlowLogProposalStore>,
    query: &LegacyFlowLogProposalQuery,
) -> Vec<LegacyFlowLogProposal> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyFlowLogProposalStore> = new_memory_legacy_flow_log_proposal_store();
        let item = new_legacy_flow_log_proposal("code-355", "name-912");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
