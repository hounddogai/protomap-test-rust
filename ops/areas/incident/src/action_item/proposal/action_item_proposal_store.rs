use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps action item proposals by code.
pub trait ActionItemProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ActionItemProposal>;
    fn save(&self, item: &ActionItemProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ActionItemProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryActionItemProposalStore {
    pub items: RefCell<BTreeMap<String, ActionItemProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl ActionItemProposalStore for MemoryActionItemProposalStore {
    fn find(&self, code: &str) -> Option<ActionItemProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ActionItemProposal) {
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

    fn list(&self) -> Vec<ActionItemProposal> {
        let mut out: Vec<ActionItemProposal> = Vec::new();
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

pub fn new_memory_action_item_proposal_store() -> Rc<dyn ActionItemProposalStore> {
    Rc::new(MemoryActionItemProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedActionItemProposalStore {
    pub inner: Rc<dyn ActionItemProposalStore>,
    pub cache: RefCell<cache::TtlCache<ActionItemProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ActionItemProposalStore for CachedActionItemProposalStore {
    fn find(&self, code: &str) -> Option<ActionItemProposal> {
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

    fn save(&self, item: &ActionItemProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ActionItemProposal> {
        self.inner.list()
    }
}

pub fn new_cached_action_item_proposal_store(
    inner: Rc<dyn ActionItemProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ActionItemProposalStore> {
    Rc::new(CachedActionItemProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ActionItemProposal>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects action item proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemProposalQuery {
    pub category: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ActionItemProposalQuery {
    pub fn matches(&self, item: &ActionItemProposal) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[ActionItemProposal]) -> Vec<ActionItemProposal> {
        let matched = collections::filter(items, |candidate: &ActionItemProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_action_item_proposals(
    store: Rc<dyn ActionItemProposalStore>,
    query: &ActionItemProposalQuery,
) -> Vec<ActionItemProposal> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ActionItemProposalStore> = new_memory_action_item_proposal_store();
        let item = new_action_item_proposal("code-988", "name-504");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
