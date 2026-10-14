use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary failover preferences by code.
pub trait PrimaryFailoverPreferenceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryFailoverPreference>;
    fn save(&self, item: &PrimaryFailoverPreference);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryFailoverPreference>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryFailoverPreferenceStore {
    pub items: RefCell<BTreeMap<String, PrimaryFailoverPreference>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryFailoverPreferenceStore for MemoryPrimaryFailoverPreferenceStore {
    fn find(&self, code: &str) -> Option<PrimaryFailoverPreference> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryFailoverPreference) {
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

    fn list(&self) -> Vec<PrimaryFailoverPreference> {
        let mut out: Vec<PrimaryFailoverPreference> = Vec::new();
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

pub fn new_memory_primary_failover_preference_store() -> Rc<dyn PrimaryFailoverPreferenceStore> {
    Rc::new(MemoryPrimaryFailoverPreferenceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryFailoverPreferenceStore {
    pub inner: Rc<dyn PrimaryFailoverPreferenceStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryFailoverPreference>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryFailoverPreferenceStore for CachedPrimaryFailoverPreferenceStore {
    fn find(&self, code: &str) -> Option<PrimaryFailoverPreference> {
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

    fn save(&self, item: &PrimaryFailoverPreference) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryFailoverPreference> {
        self.inner.list()
    }
}

pub fn new_cached_primary_failover_preference_store(
    inner: Rc<dyn PrimaryFailoverPreferenceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryFailoverPreferenceStore> {
    Rc::new(CachedPrimaryFailoverPreferenceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryFailoverPreference>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary failover preferences. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverPreferenceQuery {
    pub source: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryFailoverPreferenceQuery {
    pub fn matches(&self, item: &PrimaryFailoverPreference) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.expires_at < self.min_expires_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryFailoverPreference]) -> Vec<PrimaryFailoverPreference> {
        let matched = collections::filter(items, |candidate: &PrimaryFailoverPreference| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_failover_preferences(
    store: Rc<dyn PrimaryFailoverPreferenceStore>,
    query: &PrimaryFailoverPreferenceQuery,
) -> Vec<PrimaryFailoverPreference> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryFailoverPreferenceStore> = new_memory_primary_failover_preference_store();
        let item = new_primary_failover_preference("code-554", "title-363");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
