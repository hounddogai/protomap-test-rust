use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary limit profiles by code.
pub trait PrimaryLimitProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryLimitProfile>;
    fn save(&self, item: &PrimaryLimitProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryLimitProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryLimitProfileStore {
    pub items: RefCell<BTreeMap<String, PrimaryLimitProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryLimitProfileStore for MemoryPrimaryLimitProfileStore {
    fn find(&self, code: &str) -> Option<PrimaryLimitProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryLimitProfile) {
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

    fn list(&self) -> Vec<PrimaryLimitProfile> {
        let mut out: Vec<PrimaryLimitProfile> = Vec::new();
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

pub fn new_memory_primary_limit_profile_store() -> Rc<dyn PrimaryLimitProfileStore> {
    Rc::new(MemoryPrimaryLimitProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryLimitProfileStore {
    pub inner: Rc<dyn PrimaryLimitProfileStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryLimitProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryLimitProfileStore for CachedPrimaryLimitProfileStore {
    fn find(&self, code: &str) -> Option<PrimaryLimitProfile> {
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

    fn save(&self, item: &PrimaryLimitProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryLimitProfile> {
        self.inner.list()
    }
}

pub fn new_cached_primary_limit_profile_store(
    inner: Rc<dyn PrimaryLimitProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryLimitProfileStore> {
    Rc::new(CachedPrimaryLimitProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryLimitProfile>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary limit profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitProfileQuery {
    pub cluster: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryLimitProfileQuery {
    pub fn matches(&self, item: &PrimaryLimitProfile) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[PrimaryLimitProfile]) -> Vec<PrimaryLimitProfile> {
        let matched = collections::filter(items, |candidate: &PrimaryLimitProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_limit_profiles(
    store: Rc<dyn PrimaryLimitProfileStore>,
    query: &PrimaryLimitProfileQuery,
) -> Vec<PrimaryLimitProfile> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryLimitProfileStore> = new_memory_primary_limit_profile_store();
        let item = new_primary_limit_profile("code-385", "title-384");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
