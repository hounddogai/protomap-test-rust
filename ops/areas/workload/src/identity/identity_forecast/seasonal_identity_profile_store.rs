use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal identity profiles by code.
pub trait SeasonalIdentityProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalIdentityProfile>;
    fn save(&self, item: &SeasonalIdentityProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalIdentityProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalIdentityProfileStore {
    pub items: RefCell<BTreeMap<String, SeasonalIdentityProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalIdentityProfileStore for MemorySeasonalIdentityProfileStore {
    fn find(&self, code: &str) -> Option<SeasonalIdentityProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalIdentityProfile) {
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

    fn list(&self) -> Vec<SeasonalIdentityProfile> {
        let mut out: Vec<SeasonalIdentityProfile> = Vec::new();
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

pub fn new_memory_seasonal_identity_profile_store() -> Rc<dyn SeasonalIdentityProfileStore> {
    Rc::new(MemorySeasonalIdentityProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalIdentityProfileStore {
    pub inner: Rc<dyn SeasonalIdentityProfileStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalIdentityProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalIdentityProfileStore for CachedSeasonalIdentityProfileStore {
    fn find(&self, code: &str) -> Option<SeasonalIdentityProfile> {
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

    fn save(&self, item: &SeasonalIdentityProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalIdentityProfile> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_identity_profile_store(
    inner: Rc<dyn SeasonalIdentityProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalIdentityProfileStore> {
    Rc::new(CachedSeasonalIdentityProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalIdentityProfile>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal identity profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityProfileQuery {
    pub category: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalIdentityProfileQuery {
    pub fn matches(&self, item: &SeasonalIdentityProfile) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalIdentityProfile]) -> Vec<SeasonalIdentityProfile> {
        let matched = collections::filter(items, |candidate: &SeasonalIdentityProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_identity_profiles(
    store: Rc<dyn SeasonalIdentityProfileStore>,
    query: &SeasonalIdentityProfileQuery,
) -> Vec<SeasonalIdentityProfile> {
    query.apply(&store.list())
}
