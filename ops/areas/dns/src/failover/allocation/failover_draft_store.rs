use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps failover drafts by code.
pub trait FailoverDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FailoverDraft>;
    fn save(&self, item: &FailoverDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FailoverDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFailoverDraftStore {
    pub items: RefCell<BTreeMap<String, FailoverDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl FailoverDraftStore for MemoryFailoverDraftStore {
    fn find(&self, code: &str) -> Option<FailoverDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FailoverDraft) {
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

    fn list(&self) -> Vec<FailoverDraft> {
        let mut out: Vec<FailoverDraft> = Vec::new();
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

pub fn new_memory_failover_draft_store() -> Rc<dyn FailoverDraftStore> {
    Rc::new(MemoryFailoverDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFailoverDraftStore {
    pub inner: Rc<dyn FailoverDraftStore>,
    pub cache: RefCell<cache::TtlCache<FailoverDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FailoverDraftStore for CachedFailoverDraftStore {
    fn find(&self, code: &str) -> Option<FailoverDraft> {
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

    fn save(&self, item: &FailoverDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FailoverDraft> {
        self.inner.list()
    }
}

pub fn new_cached_failover_draft_store(
    inner: Rc<dyn FailoverDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FailoverDraftStore> {
    Rc::new(CachedFailoverDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FailoverDraft>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects failover drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverDraftQuery {
    pub cluster: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FailoverDraftQuery {
    pub fn matches(&self, item: &FailoverDraft) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FailoverDraft]) -> Vec<FailoverDraft> {
        let matched = collections::filter(items, |candidate: &FailoverDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_failover_drafts(store: Rc<dyn FailoverDraftStore>, query: &FailoverDraftQuery) -> Vec<FailoverDraft> {
    query.apply(&store.list())
}
