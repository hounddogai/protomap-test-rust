use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy nonce proposals by code.
pub trait LegacyNonceProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyNonceProposal>;
    fn save(&self, item: &LegacyNonceProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyNonceProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyNonceProposalStore {
    pub items: RefCell<BTreeMap<String, LegacyNonceProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyNonceProposalStore for MemoryLegacyNonceProposalStore {
    fn find(&self, code: &str) -> Option<LegacyNonceProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyNonceProposal) {
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

    fn list(&self) -> Vec<LegacyNonceProposal> {
        let mut out: Vec<LegacyNonceProposal> = Vec::new();
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

pub fn new_memory_legacy_nonce_proposal_store() -> Rc<dyn LegacyNonceProposalStore> {
    Rc::new(MemoryLegacyNonceProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyNonceProposalStore {
    pub inner: Rc<dyn LegacyNonceProposalStore>,
    pub cache: RefCell<cache::TtlCache<LegacyNonceProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyNonceProposalStore for CachedLegacyNonceProposalStore {
    fn find(&self, code: &str) -> Option<LegacyNonceProposal> {
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

    fn save(&self, item: &LegacyNonceProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyNonceProposal> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_nonce_proposal_store(
    inner: Rc<dyn LegacyNonceProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyNonceProposalStore> {
    Rc::new(CachedLegacyNonceProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyNonceProposal>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy nonce proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceProposalQuery {
    pub namespace: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyNonceProposalQuery {
    pub fn matches(&self, item: &LegacyNonceProposal) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyNonceProposal]) -> Vec<LegacyNonceProposal> {
        let matched = collections::filter(items, |candidate: &LegacyNonceProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_nonce_proposals(
    store: Rc<dyn LegacyNonceProposalStore>,
    query: &LegacyNonceProposalQuery,
) -> Vec<LegacyNonceProposal> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyNonceProposalStore> = new_memory_legacy_nonce_proposal_store();
        let item = new_legacy_nonce_proposal("code-362", "label-137");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
