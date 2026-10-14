use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps ttl policy profiles by code.
pub trait TtlPolicyProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TtlPolicyProfile>;
    fn save(&self, item: &TtlPolicyProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TtlPolicyProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTtlPolicyProfileStore {
    pub items: RefCell<BTreeMap<String, TtlPolicyProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl TtlPolicyProfileStore for MemoryTtlPolicyProfileStore {
    fn find(&self, code: &str) -> Option<TtlPolicyProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TtlPolicyProfile) {
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

    fn list(&self) -> Vec<TtlPolicyProfile> {
        let mut out: Vec<TtlPolicyProfile> = Vec::new();
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

pub fn new_memory_ttl_policy_profile_store() -> Rc<dyn TtlPolicyProfileStore> {
    Rc::new(MemoryTtlPolicyProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTtlPolicyProfileStore {
    pub inner: Rc<dyn TtlPolicyProfileStore>,
    pub cache: RefCell<cache::TtlCache<TtlPolicyProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TtlPolicyProfileStore for CachedTtlPolicyProfileStore {
    fn find(&self, code: &str) -> Option<TtlPolicyProfile> {
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

    fn save(&self, item: &TtlPolicyProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TtlPolicyProfile> {
        self.inner.list()
    }
}

pub fn new_cached_ttl_policy_profile_store(
    inner: Rc<dyn TtlPolicyProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TtlPolicyProfileStore> {
    Rc::new(CachedTtlPolicyProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TtlPolicyProfile>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects ttl policy profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyProfileQuery {
    pub label: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TtlPolicyProfileQuery {
    pub fn matches(&self, item: &TtlPolicyProfile) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TtlPolicyProfile]) -> Vec<TtlPolicyProfile> {
        let matched = collections::filter(items, |candidate: &TtlPolicyProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_ttl_policy_profiles(
    store: Rc<dyn TtlPolicyProfileStore>,
    query: &TtlPolicyProfileQuery,
) -> Vec<TtlPolicyProfile> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TtlPolicyProfileStore> = new_memory_ttl_policy_profile_store();
        let item = new_ttl_policy_profile("code-968", "label-664");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
