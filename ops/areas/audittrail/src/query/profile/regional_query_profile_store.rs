use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional query profiles by code.
pub trait RegionalQueryProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalQueryProfile>;
    fn save(&self, item: &RegionalQueryProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalQueryProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalQueryProfileStore {
    pub items: RefCell<BTreeMap<String, RegionalQueryProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalQueryProfileStore for MemoryRegionalQueryProfileStore {
    fn find(&self, code: &str) -> Option<RegionalQueryProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalQueryProfile) {
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

    fn list(&self) -> Vec<RegionalQueryProfile> {
        let mut out: Vec<RegionalQueryProfile> = Vec::new();
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

pub fn new_memory_regional_query_profile_store() -> Rc<dyn RegionalQueryProfileStore> {
    Rc::new(MemoryRegionalQueryProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalQueryProfileStore {
    pub inner: Rc<dyn RegionalQueryProfileStore>,
    pub cache: RefCell<cache::TtlCache<RegionalQueryProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalQueryProfileStore for CachedRegionalQueryProfileStore {
    fn find(&self, code: &str) -> Option<RegionalQueryProfile> {
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

    fn save(&self, item: &RegionalQueryProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalQueryProfile> {
        self.inner.list()
    }
}

pub fn new_cached_regional_query_profile_store(
    inner: Rc<dyn RegionalQueryProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalQueryProfileStore> {
    Rc::new(CachedRegionalQueryProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalQueryProfile>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional query profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryProfileQuery {
    pub category: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalQueryProfileQuery {
    pub fn matches(&self, item: &RegionalQueryProfile) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalQueryProfile]) -> Vec<RegionalQueryProfile> {
        let matched = collections::filter(items, |candidate: &RegionalQueryProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_query_profiles(
    store: Rc<dyn RegionalQueryProfileStore>,
    query: &RegionalQueryProfileQuery,
) -> Vec<RegionalQueryProfile> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalQueryProfileStore> = new_memory_regional_query_profile_store();
        let item = new_regional_query_profile("code-36", "name-276");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
