use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary silence profiles by code.
pub trait PrimarySilenceProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimarySilenceProfile>;
    fn save(&self, item: &PrimarySilenceProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimarySilenceProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimarySilenceProfileStore {
    pub items: RefCell<BTreeMap<String, PrimarySilenceProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimarySilenceProfileStore for MemoryPrimarySilenceProfileStore {
    fn find(&self, code: &str) -> Option<PrimarySilenceProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimarySilenceProfile) {
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

    fn list(&self) -> Vec<PrimarySilenceProfile> {
        let mut out: Vec<PrimarySilenceProfile> = Vec::new();
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

pub fn new_memory_primary_silence_profile_store() -> Rc<dyn PrimarySilenceProfileStore> {
    Rc::new(MemoryPrimarySilenceProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimarySilenceProfileStore {
    pub inner: Rc<dyn PrimarySilenceProfileStore>,
    pub cache: RefCell<cache::TtlCache<PrimarySilenceProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimarySilenceProfileStore for CachedPrimarySilenceProfileStore {
    fn find(&self, code: &str) -> Option<PrimarySilenceProfile> {
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

    fn save(&self, item: &PrimarySilenceProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimarySilenceProfile> {
        self.inner.list()
    }
}

pub fn new_cached_primary_silence_profile_store(
    inner: Rc<dyn PrimarySilenceProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimarySilenceProfileStore> {
    Rc::new(CachedPrimarySilenceProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimarySilenceProfile>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary silence profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySilenceProfileQuery {
    pub segment: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimarySilenceProfileQuery {
    pub fn matches(&self, item: &PrimarySilenceProfile) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimarySilenceProfile]) -> Vec<PrimarySilenceProfile> {
        let matched = collections::filter(items, |candidate: &PrimarySilenceProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_silence_profiles(
    store: Rc<dyn PrimarySilenceProfileStore>,
    query: &PrimarySilenceProfileQuery,
) -> Vec<PrimarySilenceProfile> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimarySilenceProfileStore> = new_memory_primary_silence_profile_store();
        let item = new_primary_silence_profile("code-775", "name-615");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
