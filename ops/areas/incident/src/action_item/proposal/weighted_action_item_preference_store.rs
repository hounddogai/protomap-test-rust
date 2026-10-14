use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted action item preferences by code.
pub trait WeightedActionItemPreferenceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedActionItemPreference>;
    fn save(&self, item: &WeightedActionItemPreference);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedActionItemPreference>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedActionItemPreferenceStore {
    pub items: RefCell<BTreeMap<String, WeightedActionItemPreference>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedActionItemPreferenceStore for MemoryWeightedActionItemPreferenceStore {
    fn find(&self, code: &str) -> Option<WeightedActionItemPreference> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedActionItemPreference) {
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

    fn list(&self) -> Vec<WeightedActionItemPreference> {
        let mut out: Vec<WeightedActionItemPreference> = Vec::new();
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

pub fn new_memory_weighted_action_item_preference_store() -> Rc<dyn WeightedActionItemPreferenceStore> {
    Rc::new(MemoryWeightedActionItemPreferenceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedActionItemPreferenceStore {
    pub inner: Rc<dyn WeightedActionItemPreferenceStore>,
    pub cache: RefCell<cache::TtlCache<WeightedActionItemPreference>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedActionItemPreferenceStore for CachedWeightedActionItemPreferenceStore {
    fn find(&self, code: &str) -> Option<WeightedActionItemPreference> {
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

    fn save(&self, item: &WeightedActionItemPreference) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedActionItemPreference> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_action_item_preference_store(
    inner: Rc<dyn WeightedActionItemPreferenceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedActionItemPreferenceStore> {
    Rc::new(CachedWeightedActionItemPreferenceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedActionItemPreference>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted action item preferences. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemPreferenceQuery {
    pub segment: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedActionItemPreferenceQuery {
    pub fn matches(&self, item: &WeightedActionItemPreference) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedActionItemPreference]) -> Vec<WeightedActionItemPreference> {
        let matched = collections::filter(items, |candidate: &WeightedActionItemPreference| {
            self.matches(candidate)
        });
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_action_item_preferences(
    store: Rc<dyn WeightedActionItemPreferenceStore>,
    query: &WeightedActionItemPreferenceQuery,
) -> Vec<WeightedActionItemPreference> {
    query.apply(&store.list())
}
