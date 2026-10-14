use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps credential summaries by code.
pub trait CredentialSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CredentialSummary>;
    fn save(&self, item: &CredentialSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CredentialSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCredentialSummaryStore {
    pub items: RefCell<BTreeMap<String, CredentialSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl CredentialSummaryStore for MemoryCredentialSummaryStore {
    fn find(&self, code: &str) -> Option<CredentialSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CredentialSummary) {
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

    fn list(&self) -> Vec<CredentialSummary> {
        let mut out: Vec<CredentialSummary> = Vec::new();
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

pub fn new_memory_credential_summary_store() -> Rc<dyn CredentialSummaryStore> {
    Rc::new(MemoryCredentialSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCredentialSummaryStore {
    pub inner: Rc<dyn CredentialSummaryStore>,
    pub cache: RefCell<cache::TtlCache<CredentialSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CredentialSummaryStore for CachedCredentialSummaryStore {
    fn find(&self, code: &str) -> Option<CredentialSummary> {
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

    fn save(&self, item: &CredentialSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CredentialSummary> {
        self.inner.list()
    }
}

pub fn new_cached_credential_summary_store(
    inner: Rc<dyn CredentialSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CredentialSummaryStore> {
    Rc::new(CachedCredentialSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CredentialSummary>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects credential summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSummaryQuery {
    pub locale: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CredentialSummaryQuery {
    pub fn matches(&self, item: &CredentialSummary) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CredentialSummary]) -> Vec<CredentialSummary> {
        let matched = collections::filter(items, |candidate: &CredentialSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_credential_summaries(
    store: Rc<dyn CredentialSummaryStore>,
    query: &CredentialSummaryQuery,
) -> Vec<CredentialSummary> {
    query.apply(&store.list())
}
