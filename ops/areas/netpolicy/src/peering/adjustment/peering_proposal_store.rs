use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps peering proposals by code.
pub trait PeeringProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PeeringProposal>;
    fn save(&self, item: &PeeringProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PeeringProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPeeringProposalStore {
    pub items: RefCell<BTreeMap<String, PeeringProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl PeeringProposalStore for MemoryPeeringProposalStore {
    fn find(&self, code: &str) -> Option<PeeringProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PeeringProposal) {
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

    fn list(&self) -> Vec<PeeringProposal> {
        let mut out: Vec<PeeringProposal> = Vec::new();
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

pub fn new_memory_peering_proposal_store() -> Rc<dyn PeeringProposalStore> {
    Rc::new(MemoryPeeringProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPeeringProposalStore {
    pub inner: Rc<dyn PeeringProposalStore>,
    pub cache: RefCell<cache::TtlCache<PeeringProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PeeringProposalStore for CachedPeeringProposalStore {
    fn find(&self, code: &str) -> Option<PeeringProposal> {
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

    fn save(&self, item: &PeeringProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PeeringProposal> {
        self.inner.list()
    }
}

pub fn new_cached_peering_proposal_store(
    inner: Rc<dyn PeeringProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PeeringProposalStore> {
    Rc::new(CachedPeeringProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PeeringProposal>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects peering proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringProposalQuery {
    pub title: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PeeringProposalQuery {
    pub fn matches(&self, item: &PeeringProposal) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PeeringProposal]) -> Vec<PeeringProposal> {
        let matched = collections::filter(items, |candidate: &PeeringProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_peering_proposals(
    store: Rc<dyn PeeringProposalStore>,
    query: &PeeringProposalQuery,
) -> Vec<PeeringProposal> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PeeringProposalStore> = new_memory_peering_proposal_store();
        let item = new_peering_proposal("code-80", "title-492");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
