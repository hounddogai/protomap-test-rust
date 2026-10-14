use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps query profiles by code.
pub trait QueryProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QueryProfile>;
    fn save(&self, item: &QueryProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QueryProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQueryProfileStore {
    pub items: RefCell<BTreeMap<String, QueryProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl QueryProfileStore for MemoryQueryProfileStore {
    fn find(&self, code: &str) -> Option<QueryProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QueryProfile) {
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

    fn list(&self) -> Vec<QueryProfile> {
        let mut out: Vec<QueryProfile> = Vec::new();
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

pub fn new_memory_query_profile_store() -> Rc<dyn QueryProfileStore> {
    Rc::new(MemoryQueryProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQueryProfileStore {
    pub inner: Rc<dyn QueryProfileStore>,
    pub cache: RefCell<cache::TtlCache<QueryProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QueryProfileStore for CachedQueryProfileStore {
    fn find(&self, code: &str) -> Option<QueryProfile> {
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

    fn save(&self, item: &QueryProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QueryProfile> {
        self.inner.list()
    }
}

pub fn new_cached_query_profile_store(
    inner: Rc<dyn QueryProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QueryProfileStore> {
    Rc::new(CachedQueryProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QueryProfile>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects query profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryProfileQuery {
    pub region: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QueryProfileQuery {
    pub fn matches(&self, item: &QueryProfile) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[QueryProfile]) -> Vec<QueryProfile> {
        let matched = collections::filter(items, |candidate: &QueryProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_query_profiles(store: Rc<dyn QueryProfileStore>, query: &QueryProfileQuery) -> Vec<QueryProfile> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn QueryProfileStore> = new_memory_query_profile_store();
        let item = new_query_profile("code-945", "name-952");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
