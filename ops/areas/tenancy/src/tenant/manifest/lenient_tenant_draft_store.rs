use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient tenant drafts by code.
pub trait LenientTenantDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientTenantDraft>;
    fn save(&self, item: &LenientTenantDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientTenantDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientTenantDraftStore {
    pub items: RefCell<BTreeMap<String, LenientTenantDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientTenantDraftStore for MemoryLenientTenantDraftStore {
    fn find(&self, code: &str) -> Option<LenientTenantDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientTenantDraft) {
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

    fn list(&self) -> Vec<LenientTenantDraft> {
        let mut out: Vec<LenientTenantDraft> = Vec::new();
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

pub fn new_memory_lenient_tenant_draft_store() -> Rc<dyn LenientTenantDraftStore> {
    Rc::new(MemoryLenientTenantDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientTenantDraftStore {
    pub inner: Rc<dyn LenientTenantDraftStore>,
    pub cache: RefCell<cache::TtlCache<LenientTenantDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientTenantDraftStore for CachedLenientTenantDraftStore {
    fn find(&self, code: &str) -> Option<LenientTenantDraft> {
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

    fn save(&self, item: &LenientTenantDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientTenantDraft> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_tenant_draft_store(
    inner: Rc<dyn LenientTenantDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientTenantDraftStore> {
    Rc::new(CachedLenientTenantDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientTenantDraft>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient tenant drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantDraftQuery {
    pub category: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientTenantDraftQuery {
    pub fn matches(&self, item: &LenientTenantDraft) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[LenientTenantDraft]) -> Vec<LenientTenantDraft> {
        let matched = collections::filter(items, |candidate: &LenientTenantDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_tenant_drafts(
    store: Rc<dyn LenientTenantDraftStore>,
    query: &LenientTenantDraftQuery,
) -> Vec<LenientTenantDraft> {
    query.apply(&store.list())
}
