use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps usage drafts by code.
pub trait UsageDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<UsageDraft>;
    fn save(&self, item: &UsageDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<UsageDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryUsageDraftStore {
    pub items: RefCell<BTreeMap<String, UsageDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl UsageDraftStore for MemoryUsageDraftStore {
    fn find(&self, code: &str) -> Option<UsageDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &UsageDraft) {
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

    fn list(&self) -> Vec<UsageDraft> {
        let mut out: Vec<UsageDraft> = Vec::new();
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

pub fn new_memory_usage_draft_store() -> Rc<dyn UsageDraftStore> {
    Rc::new(MemoryUsageDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedUsageDraftStore {
    pub inner: Rc<dyn UsageDraftStore>,
    pub cache: RefCell<cache::TtlCache<UsageDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl UsageDraftStore for CachedUsageDraftStore {
    fn find(&self, code: &str) -> Option<UsageDraft> {
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

    fn save(&self, item: &UsageDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<UsageDraft> {
        self.inner.list()
    }
}

pub fn new_cached_usage_draft_store(
    inner: Rc<dyn UsageDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn UsageDraftStore> {
    Rc::new(CachedUsageDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<UsageDraft>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects usage drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageDraftQuery {
    pub country: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl UsageDraftQuery {
    pub fn matches(&self, item: &UsageDraft) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[UsageDraft]) -> Vec<UsageDraft> {
        let matched = collections::filter(items, |candidate: &UsageDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_usage_drafts(store: Rc<dyn UsageDraftStore>, query: &UsageDraftQuery) -> Vec<UsageDraft> {
    query.apply(&store.list())
}
