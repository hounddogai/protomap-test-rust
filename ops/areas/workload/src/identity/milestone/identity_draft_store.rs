use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps identity drafts by code.
pub trait IdentityDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<IdentityDraft>;
    fn save(&self, item: &IdentityDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<IdentityDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryIdentityDraftStore {
    pub items: RefCell<BTreeMap<String, IdentityDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl IdentityDraftStore for MemoryIdentityDraftStore {
    fn find(&self, code: &str) -> Option<IdentityDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &IdentityDraft) {
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

    fn list(&self) -> Vec<IdentityDraft> {
        let mut out: Vec<IdentityDraft> = Vec::new();
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

pub fn new_memory_identity_draft_store() -> Rc<dyn IdentityDraftStore> {
    Rc::new(MemoryIdentityDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedIdentityDraftStore {
    pub inner: Rc<dyn IdentityDraftStore>,
    pub cache: RefCell<cache::TtlCache<IdentityDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl IdentityDraftStore for CachedIdentityDraftStore {
    fn find(&self, code: &str) -> Option<IdentityDraft> {
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

    fn save(&self, item: &IdentityDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<IdentityDraft> {
        self.inner.list()
    }
}

pub fn new_cached_identity_draft_store(
    inner: Rc<dyn IdentityDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn IdentityDraftStore> {
    Rc::new(CachedIdentityDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<IdentityDraft>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects identity drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityDraftQuery {
    pub cluster: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl IdentityDraftQuery {
    pub fn matches(&self, item: &IdentityDraft) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[IdentityDraft]) -> Vec<IdentityDraft> {
        let matched = collections::filter(items, |candidate: &IdentityDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_identity_drafts(store: Rc<dyn IdentityDraftStore>, query: &IdentityDraftQuery) -> Vec<IdentityDraft> {
    query.apply(&store.list())
}
