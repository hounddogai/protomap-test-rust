use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted usage preferences by code.
pub trait WeightedUsagePreferenceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedUsagePreference>;
    fn save(&self, item: &WeightedUsagePreference);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedUsagePreference>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedUsagePreferenceStore {
    pub items: RefCell<BTreeMap<String, WeightedUsagePreference>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedUsagePreferenceStore for MemoryWeightedUsagePreferenceStore {
    fn find(&self, code: &str) -> Option<WeightedUsagePreference> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedUsagePreference) {
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

    fn list(&self) -> Vec<WeightedUsagePreference> {
        let mut out: Vec<WeightedUsagePreference> = Vec::new();
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

pub fn new_memory_weighted_usage_preference_store() -> Rc<dyn WeightedUsagePreferenceStore> {
    Rc::new(MemoryWeightedUsagePreferenceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedUsagePreferenceStore {
    pub inner: Rc<dyn WeightedUsagePreferenceStore>,
    pub cache: RefCell<cache::TtlCache<WeightedUsagePreference>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedUsagePreferenceStore for CachedWeightedUsagePreferenceStore {
    fn find(&self, code: &str) -> Option<WeightedUsagePreference> {
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

    fn save(&self, item: &WeightedUsagePreference) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedUsagePreference> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_usage_preference_store(
    inner: Rc<dyn WeightedUsagePreferenceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedUsagePreferenceStore> {
    Rc::new(CachedWeightedUsagePreferenceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedUsagePreference>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted usage preferences. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsagePreferenceQuery {
    pub segment: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedUsagePreferenceQuery {
    pub fn matches(&self, item: &WeightedUsagePreference) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedUsagePreference]) -> Vec<WeightedUsagePreference> {
        let matched = collections::filter(items, |candidate: &WeightedUsagePreference| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_usage_preferences(
    store: Rc<dyn WeightedUsagePreferenceStore>,
    query: &WeightedUsagePreferenceQuery,
) -> Vec<WeightedUsagePreference> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedUsagePreferenceStore> = new_memory_weighted_usage_preference_store();
        let item = new_weighted_usage_preference("code-782", "title-648");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
