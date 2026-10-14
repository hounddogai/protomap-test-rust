use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps data export proposals by code.
pub trait DataExportProposalStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DataExportProposal>;
    fn save(&self, item: &DataExportProposal);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DataExportProposal>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDataExportProposalStore {
    pub items: RefCell<BTreeMap<String, DataExportProposal>>,
    pub order: RefCell<Vec<String>>,
}

impl DataExportProposalStore for MemoryDataExportProposalStore {
    fn find(&self, code: &str) -> Option<DataExportProposal> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DataExportProposal) {
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

    fn list(&self) -> Vec<DataExportProposal> {
        let mut out: Vec<DataExportProposal> = Vec::new();
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

pub fn new_memory_data_export_proposal_store() -> Rc<dyn DataExportProposalStore> {
    Rc::new(MemoryDataExportProposalStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDataExportProposalStore {
    pub inner: Rc<dyn DataExportProposalStore>,
    pub cache: RefCell<cache::TtlCache<DataExportProposal>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DataExportProposalStore for CachedDataExportProposalStore {
    fn find(&self, code: &str) -> Option<DataExportProposal> {
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

    fn save(&self, item: &DataExportProposal) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DataExportProposal> {
        self.inner.list()
    }
}

pub fn new_cached_data_export_proposal_store(
    inner: Rc<dyn DataExportProposalStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DataExportProposalStore> {
    Rc::new(CachedDataExportProposalStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DataExportProposal>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects data export proposals. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportProposalQuery {
    pub name: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DataExportProposalQuery {
    pub fn matches(&self, item: &DataExportProposal) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[DataExportProposal]) -> Vec<DataExportProposal> {
        let matched = collections::filter(items, |candidate: &DataExportProposal| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_data_export_proposals(
    store: Rc<dyn DataExportProposalStore>,
    query: &DataExportProposalQuery,
) -> Vec<DataExportProposal> {
    query.apply(&store.list())
}
