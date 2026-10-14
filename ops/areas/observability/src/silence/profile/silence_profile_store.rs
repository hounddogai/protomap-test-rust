use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps silence profiles by code.
pub trait SilenceProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SilenceProfile>;
    fn save(&self, item: &SilenceProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SilenceProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySilenceProfileStore {
    pub items: RefCell<BTreeMap<String, SilenceProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl SilenceProfileStore for MemorySilenceProfileStore {
    fn find(&self, code: &str) -> Option<SilenceProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SilenceProfile) {
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

    fn list(&self) -> Vec<SilenceProfile> {
        let mut out: Vec<SilenceProfile> = Vec::new();
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

pub fn new_memory_silence_profile_store() -> Rc<dyn SilenceProfileStore> {
    Rc::new(MemorySilenceProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSilenceProfileStore {
    pub inner: Rc<dyn SilenceProfileStore>,
    pub cache: RefCell<cache::TtlCache<SilenceProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SilenceProfileStore for CachedSilenceProfileStore {
    fn find(&self, code: &str) -> Option<SilenceProfile> {
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

    fn save(&self, item: &SilenceProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SilenceProfile> {
        self.inner.list()
    }
}

pub fn new_cached_silence_profile_store(
    inner: Rc<dyn SilenceProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SilenceProfileStore> {
    Rc::new(CachedSilenceProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SilenceProfile>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects silence profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceProfileQuery {
    pub category: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SilenceProfileQuery {
    pub fn matches(&self, item: &SilenceProfile) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[SilenceProfile]) -> Vec<SilenceProfile> {
        let matched = collections::filter(items, |candidate: &SilenceProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_silence_profiles(store: Rc<dyn SilenceProfileStore>, query: &SilenceProfileQuery) -> Vec<SilenceProfile> {
    query.apply(&store.list())
}
