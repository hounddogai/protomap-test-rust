use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback identity overrides by code.
pub trait FallbackIdentityOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackIdentityOverride>;
    fn save(&self, item: &FallbackIdentityOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackIdentityOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackIdentityOverrideStore {
    pub items: RefCell<BTreeMap<String, FallbackIdentityOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackIdentityOverrideStore for MemoryFallbackIdentityOverrideStore {
    fn find(&self, code: &str) -> Option<FallbackIdentityOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackIdentityOverride) {
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

    fn list(&self) -> Vec<FallbackIdentityOverride> {
        let mut out: Vec<FallbackIdentityOverride> = Vec::new();
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

pub fn new_memory_fallback_identity_override_store() -> Rc<dyn FallbackIdentityOverrideStore> {
    Rc::new(MemoryFallbackIdentityOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackIdentityOverrideStore {
    pub inner: Rc<dyn FallbackIdentityOverrideStore>,
    pub cache: RefCell<cache::TtlCache<FallbackIdentityOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackIdentityOverrideStore for CachedFallbackIdentityOverrideStore {
    fn find(&self, code: &str) -> Option<FallbackIdentityOverride> {
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

    fn save(&self, item: &FallbackIdentityOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackIdentityOverride> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_identity_override_store(
    inner: Rc<dyn FallbackIdentityOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackIdentityOverrideStore> {
    Rc::new(CachedFallbackIdentityOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackIdentityOverride>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback identity overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityOverrideQuery {
    pub name: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackIdentityOverrideQuery {
    pub fn matches(&self, item: &FallbackIdentityOverride) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackIdentityOverride]) -> Vec<FallbackIdentityOverride> {
        let matched = collections::filter(items, |candidate: &FallbackIdentityOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_identity_overrides(
    store: Rc<dyn FallbackIdentityOverrideStore>,
    query: &FallbackIdentityOverrideQuery,
) -> Vec<FallbackIdentityOverride> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackIdentityOverrideStore> = new_memory_fallback_identity_override_store();
        let item = new_fallback_identity_override("code-35", "name-987");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
