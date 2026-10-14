use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary quote profiles by code.
pub trait PrimaryQuoteProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryQuoteProfile>;
    fn save(&self, item: &PrimaryQuoteProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryQuoteProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryQuoteProfileStore {
    pub items: RefCell<BTreeMap<String, PrimaryQuoteProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryQuoteProfileStore for MemoryPrimaryQuoteProfileStore {
    fn find(&self, code: &str) -> Option<PrimaryQuoteProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryQuoteProfile) {
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

    fn list(&self) -> Vec<PrimaryQuoteProfile> {
        let mut out: Vec<PrimaryQuoteProfile> = Vec::new();
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

pub fn new_memory_primary_quote_profile_store() -> Rc<dyn PrimaryQuoteProfileStore> {
    Rc::new(MemoryPrimaryQuoteProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryQuoteProfileStore {
    pub inner: Rc<dyn PrimaryQuoteProfileStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryQuoteProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryQuoteProfileStore for CachedPrimaryQuoteProfileStore {
    fn find(&self, code: &str) -> Option<PrimaryQuoteProfile> {
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

    fn save(&self, item: &PrimaryQuoteProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryQuoteProfile> {
        self.inner.list()
    }
}

pub fn new_cached_primary_quote_profile_store(
    inner: Rc<dyn PrimaryQuoteProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryQuoteProfileStore> {
    Rc::new(CachedPrimaryQuoteProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryQuoteProfile>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary quote profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQuoteProfileQuery {
    pub source: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryQuoteProfileQuery {
    pub fn matches(&self, item: &PrimaryQuoteProfile) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryQuoteProfile]) -> Vec<PrimaryQuoteProfile> {
        let matched = collections::filter(items, |candidate: &PrimaryQuoteProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_quote_profiles(
    store: Rc<dyn PrimaryQuoteProfileStore>,
    query: &PrimaryQuoteProfileQuery,
) -> Vec<PrimaryQuoteProfile> {
    query.apply(&store.list())
}
