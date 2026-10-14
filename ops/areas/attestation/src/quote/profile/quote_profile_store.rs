use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps quote profiles by code.
pub trait QuoteProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QuoteProfile>;
    fn save(&self, item: &QuoteProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QuoteProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQuoteProfileStore {
    pub items: RefCell<BTreeMap<String, QuoteProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl QuoteProfileStore for MemoryQuoteProfileStore {
    fn find(&self, code: &str) -> Option<QuoteProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QuoteProfile) {
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

    fn list(&self) -> Vec<QuoteProfile> {
        let mut out: Vec<QuoteProfile> = Vec::new();
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

pub fn new_memory_quote_profile_store() -> Rc<dyn QuoteProfileStore> {
    Rc::new(MemoryQuoteProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQuoteProfileStore {
    pub inner: Rc<dyn QuoteProfileStore>,
    pub cache: RefCell<cache::TtlCache<QuoteProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QuoteProfileStore for CachedQuoteProfileStore {
    fn find(&self, code: &str) -> Option<QuoteProfile> {
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

    fn save(&self, item: &QuoteProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QuoteProfile> {
        self.inner.list()
    }
}

pub fn new_cached_quote_profile_store(
    inner: Rc<dyn QuoteProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QuoteProfileStore> {
    Rc::new(CachedQuoteProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QuoteProfile>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects quote profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteProfileQuery {
    pub cluster: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QuoteProfileQuery {
    pub fn matches(&self, item: &QuoteProfile) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[QuoteProfile]) -> Vec<QuoteProfile> {
        let matched = collections::filter(items, |candidate: &QuoteProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_quote_profiles(store: Rc<dyn QuoteProfileStore>, query: &QuoteProfileQuery) -> Vec<QuoteProfile> {
    query.apply(&store.list())
}
