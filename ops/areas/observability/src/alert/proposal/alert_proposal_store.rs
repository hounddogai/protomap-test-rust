use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps alert proposals by code.
pub trait AlertProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AlertProposal>;
    fn save(&self, item: &AlertProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AlertProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAlertProposalStore {
    pub items: RefCell<BTreeMap<String, AlertProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl AlertProposalStore for MemoryAlertProposalStore {
    fn find(&self, code: &str) -> Option<AlertProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AlertProposal) {
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

    fn list(&self) -> Vec<AlertProposal> {
        let mut out: Vec<AlertProposal> = Vec::new();
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

pub fn new_memory_alert_proposal_store() -> Rc<dyn AlertProposalStore> {
    Rc::new(MemoryAlertProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAlertProposalStore {
    pub inner: Rc<dyn AlertProposalStore>,
    pub cache: RefCell<cache::TtlCache<AlertProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AlertProposalStore for CachedAlertProposalStore {
    fn find(&self, code: &str) -> Option<AlertProposal> {
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

    fn save(&self, item: &AlertProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AlertProposal> {
        self.inner.list()
    }
}

pub fn new_cached_alert_proposal_store(
    inner: Rc<dyn AlertProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AlertProposalStore> {
    Rc::new(CachedAlertProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AlertProposal>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects alert proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertProposalQuery {
    pub label: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AlertProposalQuery {
    pub fn matches(&self, item: &AlertProposal) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[AlertProposal]) -> Vec<AlertProposal> {
        let matched = collections::filter(items, |candidate: &AlertProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_alert_proposals(store: Rc<dyn AlertProposalStore>, query: &AlertProposalQuery) -> Vec<AlertProposal> {
    query.apply(&store.list())
}
