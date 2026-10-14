use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual ttl policy overrides by code.
pub trait ManualTtlPolicyOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualTtlPolicyOverride>;
    fn save(&self, item: &ManualTtlPolicyOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualTtlPolicyOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualTtlPolicyOverrideStore {
    pub items: RefCell<BTreeMap<String, ManualTtlPolicyOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualTtlPolicyOverrideStore for MemoryManualTtlPolicyOverrideStore {
    fn find(&self, code: &str) -> Option<ManualTtlPolicyOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualTtlPolicyOverride) {
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

    fn list(&self) -> Vec<ManualTtlPolicyOverride> {
        let mut out: Vec<ManualTtlPolicyOverride> = Vec::new();
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

pub fn new_memory_manual_ttl_policy_override_store() -> Rc<dyn ManualTtlPolicyOverrideStore> {
    Rc::new(MemoryManualTtlPolicyOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualTtlPolicyOverrideStore {
    pub inner: Rc<dyn ManualTtlPolicyOverrideStore>,
    pub cache: RefCell<cache::TtlCache<ManualTtlPolicyOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualTtlPolicyOverrideStore for CachedManualTtlPolicyOverrideStore {
    fn find(&self, code: &str) -> Option<ManualTtlPolicyOverride> {
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

    fn save(&self, item: &ManualTtlPolicyOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualTtlPolicyOverride> {
        self.inner.list()
    }
}

pub fn new_cached_manual_ttl_policy_override_store(
    inner: Rc<dyn ManualTtlPolicyOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualTtlPolicyOverrideStore> {
    Rc::new(CachedManualTtlPolicyOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualTtlPolicyOverride>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual ttl policy overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyOverrideQuery {
    pub name: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualTtlPolicyOverrideQuery {
    pub fn matches(&self, item: &ManualTtlPolicyOverride) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[ManualTtlPolicyOverride]) -> Vec<ManualTtlPolicyOverride> {
        let matched = collections::filter(items, |candidate: &ManualTtlPolicyOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_ttl_policy_overrides(
    store: Rc<dyn ManualTtlPolicyOverrideStore>,
    query: &ManualTtlPolicyOverrideQuery,
) -> Vec<ManualTtlPolicyOverride> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualTtlPolicyOverrideStore> = new_memory_manual_ttl_policy_override_store();
        let item = new_manual_ttl_policy_override("code-118", "name-982");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
