use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global limit preferences by code.
pub trait GlobalLimitPreferenceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalLimitPreference>;
    fn save(&self, item: &GlobalLimitPreference);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalLimitPreference>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalLimitPreferenceStore {
    pub items: RefCell<BTreeMap<String, GlobalLimitPreference>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalLimitPreferenceStore for MemoryGlobalLimitPreferenceStore {
    fn find(&self, code: &str) -> Option<GlobalLimitPreference> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalLimitPreference) {
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

    fn list(&self) -> Vec<GlobalLimitPreference> {
        let mut out: Vec<GlobalLimitPreference> = Vec::new();
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

pub fn new_memory_global_limit_preference_store() -> Rc<dyn GlobalLimitPreferenceStore> {
    Rc::new(MemoryGlobalLimitPreferenceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalLimitPreferenceStore {
    pub inner: Rc<dyn GlobalLimitPreferenceStore>,
    pub cache: RefCell<cache::TtlCache<GlobalLimitPreference>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalLimitPreferenceStore for CachedGlobalLimitPreferenceStore {
    fn find(&self, code: &str) -> Option<GlobalLimitPreference> {
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

    fn save(&self, item: &GlobalLimitPreference) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalLimitPreference> {
        self.inner.list()
    }
}

pub fn new_cached_global_limit_preference_store(
    inner: Rc<dyn GlobalLimitPreferenceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalLimitPreferenceStore> {
    Rc::new(CachedGlobalLimitPreferenceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalLimitPreference>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global limit preferences. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitPreferenceQuery {
    pub title: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalLimitPreferenceQuery {
    pub fn matches(&self, item: &GlobalLimitPreference) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalLimitPreference]) -> Vec<GlobalLimitPreference> {
        let matched = collections::filter(items, |candidate: &GlobalLimitPreference| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_limit_preferences(
    store: Rc<dyn GlobalLimitPreferenceStore>,
    query: &GlobalLimitPreferenceQuery,
) -> Vec<GlobalLimitPreference> {
    query.apply(&store.list())
}
