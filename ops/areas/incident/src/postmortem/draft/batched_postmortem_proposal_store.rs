use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched postmortem proposals by code.
pub trait BatchedPostmortemProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedPostmortemProposal>;
    fn save(&self, item: &BatchedPostmortemProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedPostmortemProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedPostmortemProposalStore {
    pub items: RefCell<BTreeMap<String, BatchedPostmortemProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedPostmortemProposalStore for MemoryBatchedPostmortemProposalStore {
    fn find(&self, code: &str) -> Option<BatchedPostmortemProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedPostmortemProposal) {
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

    fn list(&self) -> Vec<BatchedPostmortemProposal> {
        let mut out: Vec<BatchedPostmortemProposal> = Vec::new();
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

pub fn new_memory_batched_postmortem_proposal_store() -> Rc<dyn BatchedPostmortemProposalStore> {
    Rc::new(MemoryBatchedPostmortemProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedPostmortemProposalStore {
    pub inner: Rc<dyn BatchedPostmortemProposalStore>,
    pub cache: RefCell<cache::TtlCache<BatchedPostmortemProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedPostmortemProposalStore for CachedBatchedPostmortemProposalStore {
    fn find(&self, code: &str) -> Option<BatchedPostmortemProposal> {
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

    fn save(&self, item: &BatchedPostmortemProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedPostmortemProposal> {
        self.inner.list()
    }
}

pub fn new_cached_batched_postmortem_proposal_store(
    inner: Rc<dyn BatchedPostmortemProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedPostmortemProposalStore> {
    Rc::new(CachedBatchedPostmortemProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedPostmortemProposal>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched postmortem proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemProposalQuery {
    pub label: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedPostmortemProposalQuery {
    pub fn matches(&self, item: &BatchedPostmortemProposal) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedPostmortemProposal]) -> Vec<BatchedPostmortemProposal> {
        let matched = collections::filter(items, |candidate: &BatchedPostmortemProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_postmortem_proposals(
    store: Rc<dyn BatchedPostmortemProposalStore>,
    query: &BatchedPostmortemProposalQuery,
) -> Vec<BatchedPostmortemProposal> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedPostmortemProposalStore> = new_memory_batched_postmortem_proposal_store();
        let item = new_batched_postmortem_proposal("code-176", "label-559");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
