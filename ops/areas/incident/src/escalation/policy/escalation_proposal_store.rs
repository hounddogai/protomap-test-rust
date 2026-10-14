use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps escalation proposals by code.
pub trait EscalationProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EscalationProposal>;
    fn save(&self, item: &EscalationProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EscalationProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEscalationProposalStore {
    pub items: RefCell<BTreeMap<String, EscalationProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl EscalationProposalStore for MemoryEscalationProposalStore {
    fn find(&self, code: &str) -> Option<EscalationProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EscalationProposal) {
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

    fn list(&self) -> Vec<EscalationProposal> {
        let mut out: Vec<EscalationProposal> = Vec::new();
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

pub fn new_memory_escalation_proposal_store() -> Rc<dyn EscalationProposalStore> {
    Rc::new(MemoryEscalationProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEscalationProposalStore {
    pub inner: Rc<dyn EscalationProposalStore>,
    pub cache: RefCell<cache::TtlCache<EscalationProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EscalationProposalStore for CachedEscalationProposalStore {
    fn find(&self, code: &str) -> Option<EscalationProposal> {
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

    fn save(&self, item: &EscalationProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EscalationProposal> {
        self.inner.list()
    }
}

pub fn new_cached_escalation_proposal_store(
    inner: Rc<dyn EscalationProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EscalationProposalStore> {
    Rc::new(CachedEscalationProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EscalationProposal>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects escalation proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationProposalQuery {
    pub scope: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EscalationProposalQuery {
    pub fn matches(&self, item: &EscalationProposal) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[EscalationProposal]) -> Vec<EscalationProposal> {
        let matched = collections::filter(items, |candidate: &EscalationProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_escalation_proposals(
    store: Rc<dyn EscalationProposalStore>,
    query: &EscalationProposalQuery,
) -> Vec<EscalationProposal> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn EscalationProposalStore> = new_memory_escalation_proposal_store();
        let item = new_escalation_proposal("code-667", "title-232");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
