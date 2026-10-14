use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps allowlist profiles by code.
pub trait AllowlistProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AllowlistProfile>;
    fn save(&self, item: &AllowlistProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AllowlistProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAllowlistProfileStore {
    pub items: RefCell<BTreeMap<String, AllowlistProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl AllowlistProfileStore for MemoryAllowlistProfileStore {
    fn find(&self, code: &str) -> Option<AllowlistProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AllowlistProfile) {
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

    fn list(&self) -> Vec<AllowlistProfile> {
        let mut out: Vec<AllowlistProfile> = Vec::new();
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

pub fn new_memory_allowlist_profile_store() -> Rc<dyn AllowlistProfileStore> {
    Rc::new(MemoryAllowlistProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAllowlistProfileStore {
    pub inner: Rc<dyn AllowlistProfileStore>,
    pub cache: RefCell<cache::TtlCache<AllowlistProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AllowlistProfileStore for CachedAllowlistProfileStore {
    fn find(&self, code: &str) -> Option<AllowlistProfile> {
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

    fn save(&self, item: &AllowlistProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AllowlistProfile> {
        self.inner.list()
    }
}

pub fn new_cached_allowlist_profile_store(
    inner: Rc<dyn AllowlistProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AllowlistProfileStore> {
    Rc::new(CachedAllowlistProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AllowlistProfile>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects allowlist profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistProfileQuery {
    pub title: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AllowlistProfileQuery {
    pub fn matches(&self, item: &AllowlistProfile) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AllowlistProfile]) -> Vec<AllowlistProfile> {
        let matched = collections::filter(items, |candidate: &AllowlistProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_allowlist_profiles(
    store: Rc<dyn AllowlistProfileStore>,
    query: &AllowlistProfileQuery,
) -> Vec<AllowlistProfile> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AllowlistProfileStore> = new_memory_allowlist_profile_store();
        let item = new_allowlist_profile("code-894", "title-625");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
