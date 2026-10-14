use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps trust domain drafts by code.
pub trait TrustDomainDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TrustDomainDraft>;
    fn save(&self, item: &TrustDomainDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TrustDomainDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTrustDomainDraftStore {
    pub items: RefCell<BTreeMap<String, TrustDomainDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl TrustDomainDraftStore for MemoryTrustDomainDraftStore {
    fn find(&self, code: &str) -> Option<TrustDomainDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TrustDomainDraft) {
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

    fn list(&self) -> Vec<TrustDomainDraft> {
        let mut out: Vec<TrustDomainDraft> = Vec::new();
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

pub fn new_memory_trust_domain_draft_store() -> Rc<dyn TrustDomainDraftStore> {
    Rc::new(MemoryTrustDomainDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTrustDomainDraftStore {
    pub inner: Rc<dyn TrustDomainDraftStore>,
    pub cache: RefCell<cache::TtlCache<TrustDomainDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TrustDomainDraftStore for CachedTrustDomainDraftStore {
    fn find(&self, code: &str) -> Option<TrustDomainDraft> {
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

    fn save(&self, item: &TrustDomainDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TrustDomainDraft> {
        self.inner.list()
    }
}

pub fn new_cached_trust_domain_draft_store(
    inner: Rc<dyn TrustDomainDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TrustDomainDraftStore> {
    Rc::new(CachedTrustDomainDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TrustDomainDraft>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects trust domain drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainDraftQuery {
    pub region: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TrustDomainDraftQuery {
    pub fn matches(&self, item: &TrustDomainDraft) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TrustDomainDraft]) -> Vec<TrustDomainDraft> {
        let matched = collections::filter(items, |candidate: &TrustDomainDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_trust_domain_drafts(
    store: Rc<dyn TrustDomainDraftStore>,
    query: &TrustDomainDraftQuery,
) -> Vec<TrustDomainDraft> {
    query.apply(&store.list())
}
