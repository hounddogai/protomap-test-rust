use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy query preferences by code.
pub trait LegacyQueryPreferenceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyQueryPreference>;
    fn save(&self, item: &LegacyQueryPreference);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyQueryPreference>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyQueryPreferenceStore {
    pub items: RefCell<BTreeMap<String, LegacyQueryPreference>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyQueryPreferenceStore for MemoryLegacyQueryPreferenceStore {
    fn find(&self, code: &str) -> Option<LegacyQueryPreference> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyQueryPreference) {
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

    fn list(&self) -> Vec<LegacyQueryPreference> {
        let mut out: Vec<LegacyQueryPreference> = Vec::new();
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

pub fn new_memory_legacy_query_preference_store() -> Rc<dyn LegacyQueryPreferenceStore> {
    Rc::new(MemoryLegacyQueryPreferenceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyQueryPreferenceStore {
    pub inner: Rc<dyn LegacyQueryPreferenceStore>,
    pub cache: RefCell<cache::TtlCache<LegacyQueryPreference>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyQueryPreferenceStore for CachedLegacyQueryPreferenceStore {
    fn find(&self, code: &str) -> Option<LegacyQueryPreference> {
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

    fn save(&self, item: &LegacyQueryPreference) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyQueryPreference> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_query_preference_store(
    inner: Rc<dyn LegacyQueryPreferenceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyQueryPreferenceStore> {
    Rc::new(CachedLegacyQueryPreferenceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyQueryPreference>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy query preferences. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQueryPreferenceQuery {
    pub scope: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyQueryPreferenceQuery {
    pub fn matches(&self, item: &LegacyQueryPreference) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyQueryPreference]) -> Vec<LegacyQueryPreference> {
        let matched = collections::filter(items, |candidate: &LegacyQueryPreference| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_query_preferences(
    store: Rc<dyn LegacyQueryPreferenceStore>,
    query: &LegacyQueryPreferenceQuery,
) -> Vec<LegacyQueryPreference> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyQueryPreferenceStore> = new_memory_legacy_query_preference_store();
        let item = new_legacy_query_preference("code-627", "label-618");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
