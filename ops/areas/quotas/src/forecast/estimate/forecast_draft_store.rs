use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps forecast drafts by code.
pub trait ForecastDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ForecastDraft>;
    fn save(&self, item: &ForecastDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ForecastDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryForecastDraftStore {
    pub items: RefCell<BTreeMap<String, ForecastDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl ForecastDraftStore for MemoryForecastDraftStore {
    fn find(&self, code: &str) -> Option<ForecastDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ForecastDraft) {
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

    fn list(&self) -> Vec<ForecastDraft> {
        let mut out: Vec<ForecastDraft> = Vec::new();
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

pub fn new_memory_forecast_draft_store() -> Rc<dyn ForecastDraftStore> {
    Rc::new(MemoryForecastDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedForecastDraftStore {
    pub inner: Rc<dyn ForecastDraftStore>,
    pub cache: RefCell<cache::TtlCache<ForecastDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ForecastDraftStore for CachedForecastDraftStore {
    fn find(&self, code: &str) -> Option<ForecastDraft> {
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

    fn save(&self, item: &ForecastDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ForecastDraft> {
        self.inner.list()
    }
}

pub fn new_cached_forecast_draft_store(
    inner: Rc<dyn ForecastDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ForecastDraftStore> {
    Rc::new(CachedForecastDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ForecastDraft>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects forecast drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastDraftQuery {
    pub name: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ForecastDraftQuery {
    pub fn matches(&self, item: &ForecastDraft) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[ForecastDraft]) -> Vec<ForecastDraft> {
        let matched = collections::filter(items, |candidate: &ForecastDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_forecast_drafts(store: Rc<dyn ForecastDraftStore>, query: &ForecastDraftQuery) -> Vec<ForecastDraft> {
    query.apply(&store.list())
}
