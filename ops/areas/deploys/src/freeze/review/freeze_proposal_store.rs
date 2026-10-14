use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps freeze proposals by code.
pub trait FreezeProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FreezeProposal>;
    fn save(&self, item: &FreezeProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FreezeProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFreezeProposalStore {
    pub items: RefCell<BTreeMap<String, FreezeProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl FreezeProposalStore for MemoryFreezeProposalStore {
    fn find(&self, code: &str) -> Option<FreezeProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FreezeProposal) {
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

    fn list(&self) -> Vec<FreezeProposal> {
        let mut out: Vec<FreezeProposal> = Vec::new();
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

pub fn new_memory_freeze_proposal_store() -> Rc<dyn FreezeProposalStore> {
    Rc::new(MemoryFreezeProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFreezeProposalStore {
    pub inner: Rc<dyn FreezeProposalStore>,
    pub cache: RefCell<cache::TtlCache<FreezeProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FreezeProposalStore for CachedFreezeProposalStore {
    fn find(&self, code: &str) -> Option<FreezeProposal> {
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

    fn save(&self, item: &FreezeProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FreezeProposal> {
        self.inner.list()
    }
}

pub fn new_cached_freeze_proposal_store(
    inner: Rc<dyn FreezeProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FreezeProposalStore> {
    Rc::new(CachedFreezeProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FreezeProposal>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects freeze proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeProposalQuery {
    pub source: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FreezeProposalQuery {
    pub fn matches(&self, item: &FreezeProposal) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FreezeProposal]) -> Vec<FreezeProposal> {
        let matched = collections::filter(items, |candidate: &FreezeProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_freeze_proposals(store: Rc<dyn FreezeProposalStore>, query: &FreezeProposalQuery) -> Vec<FreezeProposal> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FreezeProposalStore> = new_memory_freeze_proposal_store();
        let item = new_freeze_proposal("code-81", "label-297");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
