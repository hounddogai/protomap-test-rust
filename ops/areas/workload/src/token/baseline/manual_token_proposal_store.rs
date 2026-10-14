use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual token proposals by code.
pub trait ManualTokenProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualTokenProposal>;
    fn save(&self, item: &ManualTokenProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualTokenProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualTokenProposalStore {
    pub items: RefCell<BTreeMap<String, ManualTokenProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualTokenProposalStore for MemoryManualTokenProposalStore {
    fn find(&self, code: &str) -> Option<ManualTokenProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualTokenProposal) {
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

    fn list(&self) -> Vec<ManualTokenProposal> {
        let mut out: Vec<ManualTokenProposal> = Vec::new();
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

pub fn new_memory_manual_token_proposal_store() -> Rc<dyn ManualTokenProposalStore> {
    Rc::new(MemoryManualTokenProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualTokenProposalStore {
    pub inner: Rc<dyn ManualTokenProposalStore>,
    pub cache: RefCell<cache::TtlCache<ManualTokenProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualTokenProposalStore for CachedManualTokenProposalStore {
    fn find(&self, code: &str) -> Option<ManualTokenProposal> {
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

    fn save(&self, item: &ManualTokenProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualTokenProposal> {
        self.inner.list()
    }
}

pub fn new_cached_manual_token_proposal_store(
    inner: Rc<dyn ManualTokenProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualTokenProposalStore> {
    Rc::new(CachedManualTokenProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualTokenProposal>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual token proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTokenProposalQuery {
    pub cluster: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualTokenProposalQuery {
    pub fn matches(&self, item: &ManualTokenProposal) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualTokenProposal]) -> Vec<ManualTokenProposal> {
        let matched = collections::filter(items, |candidate: &ManualTokenProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_token_proposals(
    store: Rc<dyn ManualTokenProposalStore>,
    query: &ManualTokenProposalQuery,
) -> Vec<ManualTokenProposal> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualTokenProposalStore> = new_memory_manual_token_proposal_store();
        let item = new_manual_token_proposal("code-778", "name-515");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
