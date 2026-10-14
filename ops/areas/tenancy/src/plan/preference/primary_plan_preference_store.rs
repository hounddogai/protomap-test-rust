use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary plan preferences by code.
pub trait PrimaryPlanPreferenceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryPlanPreference>;
    fn save(&self, item: &PrimaryPlanPreference);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryPlanPreference>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryPlanPreferenceStore {
    pub items: RefCell<BTreeMap<String, PrimaryPlanPreference>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryPlanPreferenceStore for MemoryPrimaryPlanPreferenceStore {
    fn find(&self, code: &str) -> Option<PrimaryPlanPreference> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryPlanPreference) {
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

    fn list(&self) -> Vec<PrimaryPlanPreference> {
        let mut out: Vec<PrimaryPlanPreference> = Vec::new();
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

pub fn new_memory_primary_plan_preference_store() -> Rc<dyn PrimaryPlanPreferenceStore> {
    Rc::new(MemoryPrimaryPlanPreferenceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryPlanPreferenceStore {
    pub inner: Rc<dyn PrimaryPlanPreferenceStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryPlanPreference>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryPlanPreferenceStore for CachedPrimaryPlanPreferenceStore {
    fn find(&self, code: &str) -> Option<PrimaryPlanPreference> {
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

    fn save(&self, item: &PrimaryPlanPreference) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryPlanPreference> {
        self.inner.list()
    }
}

pub fn new_cached_primary_plan_preference_store(
    inner: Rc<dyn PrimaryPlanPreferenceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryPlanPreferenceStore> {
    Rc::new(CachedPrimaryPlanPreferenceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryPlanPreference>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary plan preferences. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanPreferenceQuery {
    pub name: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryPlanPreferenceQuery {
    pub fn matches(&self, item: &PrimaryPlanPreference) -> bool {
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

    pub fn apply(&self, items: &[PrimaryPlanPreference]) -> Vec<PrimaryPlanPreference> {
        let matched = collections::filter(items, |candidate: &PrimaryPlanPreference| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_plan_preferences(
    store: Rc<dyn PrimaryPlanPreferenceStore>,
    query: &PrimaryPlanPreferenceQuery,
) -> Vec<PrimaryPlanPreference> {
    query.apply(&store.list())
}
