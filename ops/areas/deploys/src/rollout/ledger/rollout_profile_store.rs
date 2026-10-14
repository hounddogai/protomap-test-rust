use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rollout profiles by code.
pub trait RolloutProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RolloutProfile>;
    fn save(&self, item: &RolloutProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RolloutProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRolloutProfileStore {
    pub items: RefCell<BTreeMap<String, RolloutProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl RolloutProfileStore for MemoryRolloutProfileStore {
    fn find(&self, code: &str) -> Option<RolloutProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RolloutProfile) {
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

    fn list(&self) -> Vec<RolloutProfile> {
        let mut out: Vec<RolloutProfile> = Vec::new();
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

pub fn new_memory_rollout_profile_store() -> Rc<dyn RolloutProfileStore> {
    Rc::new(MemoryRolloutProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRolloutProfileStore {
    pub inner: Rc<dyn RolloutProfileStore>,
    pub cache: RefCell<cache::TtlCache<RolloutProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RolloutProfileStore for CachedRolloutProfileStore {
    fn find(&self, code: &str) -> Option<RolloutProfile> {
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

    fn save(&self, item: &RolloutProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RolloutProfile> {
        self.inner.list()
    }
}

pub fn new_cached_rollout_profile_store(
    inner: Rc<dyn RolloutProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RolloutProfileStore> {
    Rc::new(CachedRolloutProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RolloutProfile>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rollout profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutProfileQuery {
    pub name: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RolloutProfileQuery {
    pub fn matches(&self, item: &RolloutProfile) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RolloutProfile]) -> Vec<RolloutProfile> {
        let matched = collections::filter(items, |candidate: &RolloutProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rollout_profiles(store: Rc<dyn RolloutProfileStore>, query: &RolloutProfileQuery) -> Vec<RolloutProfile> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RolloutProfileStore> = new_memory_rollout_profile_store();
        let item = new_rollout_profile("code-925", "name-144");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
