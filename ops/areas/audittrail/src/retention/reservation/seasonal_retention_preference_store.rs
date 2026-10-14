use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal retention preferences by code.
pub trait SeasonalRetentionPreferenceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalRetentionPreference>;
    fn save(&self, item: &SeasonalRetentionPreference);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalRetentionPreference>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalRetentionPreferenceStore {
    pub items: RefCell<BTreeMap<String, SeasonalRetentionPreference>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalRetentionPreferenceStore for MemorySeasonalRetentionPreferenceStore {
    fn find(&self, code: &str) -> Option<SeasonalRetentionPreference> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalRetentionPreference) {
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

    fn list(&self) -> Vec<SeasonalRetentionPreference> {
        let mut out: Vec<SeasonalRetentionPreference> = Vec::new();
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

pub fn new_memory_seasonal_retention_preference_store() -> Rc<dyn SeasonalRetentionPreferenceStore> {
    Rc::new(MemorySeasonalRetentionPreferenceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalRetentionPreferenceStore {
    pub inner: Rc<dyn SeasonalRetentionPreferenceStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalRetentionPreference>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalRetentionPreferenceStore for CachedSeasonalRetentionPreferenceStore {
    fn find(&self, code: &str) -> Option<SeasonalRetentionPreference> {
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

    fn save(&self, item: &SeasonalRetentionPreference) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalRetentionPreference> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_retention_preference_store(
    inner: Rc<dyn SeasonalRetentionPreferenceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalRetentionPreferenceStore> {
    Rc::new(CachedSeasonalRetentionPreferenceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalRetentionPreference>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal retention preferences. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionPreferenceQuery {
    pub scope: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalRetentionPreferenceQuery {
    pub fn matches(&self, item: &SeasonalRetentionPreference) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalRetentionPreference]) -> Vec<SeasonalRetentionPreference> {
        let matched = collections::filter(items, |candidate: &SeasonalRetentionPreference| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_retention_preferences(
    store: Rc<dyn SeasonalRetentionPreferenceStore>,
    query: &SeasonalRetentionPreferenceQuery,
) -> Vec<SeasonalRetentionPreference> {
    query.apply(&store.list())
}
