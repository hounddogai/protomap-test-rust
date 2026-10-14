use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback runbook proposals by code.
pub trait FallbackRunbookProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackRunbookProposal>;
    fn save(&self, item: &FallbackRunbookProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackRunbookProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackRunbookProposalStore {
    pub items: RefCell<BTreeMap<String, FallbackRunbookProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackRunbookProposalStore for MemoryFallbackRunbookProposalStore {
    fn find(&self, code: &str) -> Option<FallbackRunbookProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackRunbookProposal) {
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

    fn list(&self) -> Vec<FallbackRunbookProposal> {
        let mut out: Vec<FallbackRunbookProposal> = Vec::new();
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

pub fn new_memory_fallback_runbook_proposal_store() -> Rc<dyn FallbackRunbookProposalStore> {
    Rc::new(MemoryFallbackRunbookProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackRunbookProposalStore {
    pub inner: Rc<dyn FallbackRunbookProposalStore>,
    pub cache: RefCell<cache::TtlCache<FallbackRunbookProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackRunbookProposalStore for CachedFallbackRunbookProposalStore {
    fn find(&self, code: &str) -> Option<FallbackRunbookProposal> {
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

    fn save(&self, item: &FallbackRunbookProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackRunbookProposal> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_runbook_proposal_store(
    inner: Rc<dyn FallbackRunbookProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackRunbookProposalStore> {
    Rc::new(CachedFallbackRunbookProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackRunbookProposal>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback runbook proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRunbookProposalQuery {
    pub country: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackRunbookProposalQuery {
    pub fn matches(&self, item: &FallbackRunbookProposal) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackRunbookProposal]) -> Vec<FallbackRunbookProposal> {
        let matched = collections::filter(items, |candidate: &FallbackRunbookProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_runbook_proposals(
    store: Rc<dyn FallbackRunbookProposalStore>,
    query: &FallbackRunbookProposalQuery,
) -> Vec<FallbackRunbookProposal> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackRunbookProposalStore> = new_memory_fallback_runbook_proposal_store();
        let item = new_fallback_runbook_proposal("code-492", "label-522");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
