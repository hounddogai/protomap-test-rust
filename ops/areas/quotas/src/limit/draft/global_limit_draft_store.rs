use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global limit drafts by code.
pub trait GlobalLimitDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalLimitDraft>;
    fn save(&self, item: &GlobalLimitDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalLimitDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalLimitDraftStore {
    pub items: RefCell<BTreeMap<String, GlobalLimitDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalLimitDraftStore for MemoryGlobalLimitDraftStore {
    fn find(&self, code: &str) -> Option<GlobalLimitDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalLimitDraft) {
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

    fn list(&self) -> Vec<GlobalLimitDraft> {
        let mut out: Vec<GlobalLimitDraft> = Vec::new();
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

pub fn new_memory_global_limit_draft_store() -> Rc<dyn GlobalLimitDraftStore> {
    Rc::new(MemoryGlobalLimitDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalLimitDraftStore {
    pub inner: Rc<dyn GlobalLimitDraftStore>,
    pub cache: RefCell<cache::TtlCache<GlobalLimitDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalLimitDraftStore for CachedGlobalLimitDraftStore {
    fn find(&self, code: &str) -> Option<GlobalLimitDraft> {
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

    fn save(&self, item: &GlobalLimitDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalLimitDraft> {
        self.inner.list()
    }
}

pub fn new_cached_global_limit_draft_store(
    inner: Rc<dyn GlobalLimitDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalLimitDraftStore> {
    Rc::new(CachedGlobalLimitDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalLimitDraft>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global limit drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitDraftQuery {
    pub scope: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalLimitDraftQuery {
    pub fn matches(&self, item: &GlobalLimitDraft) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalLimitDraft]) -> Vec<GlobalLimitDraft> {
        let matched = collections::filter(items, |candidate: &GlobalLimitDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_limit_drafts(
    store: Rc<dyn GlobalLimitDraftStore>,
    query: &GlobalLimitDraftQuery,
) -> Vec<GlobalLimitDraft> {
    query.apply(&store.list())
}
