use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps evidence entries by code.
pub trait EvidenceEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EvidenceEntry>;
    fn save(&self, item: &EvidenceEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EvidenceEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEvidenceEntryStore {
    pub items: RefCell<BTreeMap<String, EvidenceEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl EvidenceEntryStore for MemoryEvidenceEntryStore {
    fn find(&self, code: &str) -> Option<EvidenceEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EvidenceEntry) {
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

    fn list(&self) -> Vec<EvidenceEntry> {
        let mut out: Vec<EvidenceEntry> = Vec::new();
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

pub fn new_memory_evidence_entry_store() -> Rc<dyn EvidenceEntryStore> {
    Rc::new(MemoryEvidenceEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEvidenceEntryStore {
    pub inner: Rc<dyn EvidenceEntryStore>,
    pub cache: RefCell<cache::TtlCache<EvidenceEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EvidenceEntryStore for CachedEvidenceEntryStore {
    fn find(&self, code: &str) -> Option<EvidenceEntry> {
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

    fn save(&self, item: &EvidenceEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EvidenceEntry> {
        self.inner.list()
    }
}

pub fn new_cached_evidence_entry_store(
    inner: Rc<dyn EvidenceEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EvidenceEntryStore> {
    Rc::new(CachedEvidenceEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EvidenceEntry>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects evidence entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceEntryQuery {
    pub title: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EvidenceEntryQuery {
    pub fn matches(&self, item: &EvidenceEntry) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[EvidenceEntry]) -> Vec<EvidenceEntry> {
        let matched = collections::filter(items, |candidate: &EvidenceEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_evidence_entries(store: Rc<dyn EvidenceEntryStore>, query: &EvidenceEntryQuery) -> Vec<EvidenceEntry> {
    query.apply(&store.list())
}
