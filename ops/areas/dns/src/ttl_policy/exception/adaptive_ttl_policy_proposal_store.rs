use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive ttl policy proposals by code.
pub trait AdaptiveTtlPolicyProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveTtlPolicyProposal>;
    fn save(&self, item: &AdaptiveTtlPolicyProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveTtlPolicyProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveTtlPolicyProposalStore {
    pub items: RefCell<BTreeMap<String, AdaptiveTtlPolicyProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveTtlPolicyProposalStore for MemoryAdaptiveTtlPolicyProposalStore {
    fn find(&self, code: &str) -> Option<AdaptiveTtlPolicyProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveTtlPolicyProposal) {
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

    fn list(&self) -> Vec<AdaptiveTtlPolicyProposal> {
        let mut out: Vec<AdaptiveTtlPolicyProposal> = Vec::new();
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

pub fn new_memory_adaptive_ttl_policy_proposal_store() -> Rc<dyn AdaptiveTtlPolicyProposalStore> {
    Rc::new(MemoryAdaptiveTtlPolicyProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveTtlPolicyProposalStore {
    pub inner: Rc<dyn AdaptiveTtlPolicyProposalStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveTtlPolicyProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveTtlPolicyProposalStore for CachedAdaptiveTtlPolicyProposalStore {
    fn find(&self, code: &str) -> Option<AdaptiveTtlPolicyProposal> {
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

    fn save(&self, item: &AdaptiveTtlPolicyProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveTtlPolicyProposal> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_ttl_policy_proposal_store(
    inner: Rc<dyn AdaptiveTtlPolicyProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveTtlPolicyProposalStore> {
    Rc::new(CachedAdaptiveTtlPolicyProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveTtlPolicyProposal>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive ttl policy proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTtlPolicyProposalQuery {
    pub label: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveTtlPolicyProposalQuery {
    pub fn matches(&self, item: &AdaptiveTtlPolicyProposal) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveTtlPolicyProposal]) -> Vec<AdaptiveTtlPolicyProposal> {
        let matched = collections::filter(items, |candidate: &AdaptiveTtlPolicyProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_ttl_policy_proposals(
    store: Rc<dyn AdaptiveTtlPolicyProposalStore>,
    query: &AdaptiveTtlPolicyProposalQuery,
) -> Vec<AdaptiveTtlPolicyProposal> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AdaptiveTtlPolicyProposalStore> = new_memory_adaptive_ttl_policy_proposal_store();
        let item = new_adaptive_ttl_policy_proposal("code-188", "label-413");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
