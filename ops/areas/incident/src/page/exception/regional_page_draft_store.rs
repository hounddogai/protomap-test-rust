use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional page drafts by code.
pub trait RegionalPageDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalPageDraft>;
    fn save(&self, item: &RegionalPageDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalPageDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalPageDraftStore {
    pub items: RefCell<BTreeMap<String, RegionalPageDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalPageDraftStore for MemoryRegionalPageDraftStore {
    fn find(&self, code: &str) -> Option<RegionalPageDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalPageDraft) {
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

    fn list(&self) -> Vec<RegionalPageDraft> {
        let mut out: Vec<RegionalPageDraft> = Vec::new();
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

pub fn new_memory_regional_page_draft_store() -> Rc<dyn RegionalPageDraftStore> {
    Rc::new(MemoryRegionalPageDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalPageDraftStore {
    pub inner: Rc<dyn RegionalPageDraftStore>,
    pub cache: RefCell<cache::TtlCache<RegionalPageDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalPageDraftStore for CachedRegionalPageDraftStore {
    fn find(&self, code: &str) -> Option<RegionalPageDraft> {
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

    fn save(&self, item: &RegionalPageDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalPageDraft> {
        self.inner.list()
    }
}

pub fn new_cached_regional_page_draft_store(
    inner: Rc<dyn RegionalPageDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalPageDraftStore> {
    Rc::new(CachedRegionalPageDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalPageDraft>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional page drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageDraftQuery {
    pub title: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalPageDraftQuery {
    pub fn matches(&self, item: &RegionalPageDraft) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalPageDraft]) -> Vec<RegionalPageDraft> {
        let matched = collections::filter(items, |candidate: &RegionalPageDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_page_drafts(
    store: Rc<dyn RegionalPageDraftStore>,
    query: &RegionalPageDraftQuery,
) -> Vec<RegionalPageDraft> {
    query.apply(&store.list())
}
