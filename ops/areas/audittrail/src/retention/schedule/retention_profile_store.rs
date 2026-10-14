use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps retention profiles by code.
pub trait RetentionProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RetentionProfile>;
    fn save(&self, item: &RetentionProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RetentionProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRetentionProfileStore {
    pub items: RefCell<BTreeMap<String, RetentionProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl RetentionProfileStore for MemoryRetentionProfileStore {
    fn find(&self, code: &str) -> Option<RetentionProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RetentionProfile) {
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

    fn list(&self) -> Vec<RetentionProfile> {
        let mut out: Vec<RetentionProfile> = Vec::new();
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

pub fn new_memory_retention_profile_store() -> Rc<dyn RetentionProfileStore> {
    Rc::new(MemoryRetentionProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRetentionProfileStore {
    pub inner: Rc<dyn RetentionProfileStore>,
    pub cache: RefCell<cache::TtlCache<RetentionProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RetentionProfileStore for CachedRetentionProfileStore {
    fn find(&self, code: &str) -> Option<RetentionProfile> {
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

    fn save(&self, item: &RetentionProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RetentionProfile> {
        self.inner.list()
    }
}

pub fn new_cached_retention_profile_store(
    inner: Rc<dyn RetentionProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RetentionProfileStore> {
    Rc::new(CachedRetentionProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RetentionProfile>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects retention profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionProfileQuery {
    pub scope: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RetentionProfileQuery {
    pub fn matches(&self, item: &RetentionProfile) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RetentionProfile]) -> Vec<RetentionProfile> {
        let matched = collections::filter(items, |candidate: &RetentionProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_retention_profiles(
    store: Rc<dyn RetentionProfileStore>,
    query: &RetentionProfileQuery,
) -> Vec<RetentionProfile> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RetentionProfileStore> = new_memory_retention_profile_store();
        let item = new_retention_profile("code-186", "title-107");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
