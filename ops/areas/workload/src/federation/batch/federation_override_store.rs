use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps federation overrides by code.
pub trait FederationOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FederationOverride>;
    fn save(&self, item: &FederationOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FederationOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFederationOverrideStore {
    pub items: RefCell<BTreeMap<String, FederationOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl FederationOverrideStore for MemoryFederationOverrideStore {
    fn find(&self, code: &str) -> Option<FederationOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FederationOverride) {
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

    fn list(&self) -> Vec<FederationOverride> {
        let mut out: Vec<FederationOverride> = Vec::new();
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

pub fn new_memory_federation_override_store() -> Rc<dyn FederationOverrideStore> {
    Rc::new(MemoryFederationOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFederationOverrideStore {
    pub inner: Rc<dyn FederationOverrideStore>,
    pub cache: RefCell<cache::TtlCache<FederationOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FederationOverrideStore for CachedFederationOverrideStore {
    fn find(&self, code: &str) -> Option<FederationOverride> {
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

    fn save(&self, item: &FederationOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FederationOverride> {
        self.inner.list()
    }
}

pub fn new_cached_federation_override_store(
    inner: Rc<dyn FederationOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FederationOverrideStore> {
    Rc::new(CachedFederationOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FederationOverride>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects federation overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationOverrideQuery {
    pub title: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FederationOverrideQuery {
    pub fn matches(&self, item: &FederationOverride) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FederationOverride]) -> Vec<FederationOverride> {
        let matched = collections::filter(items, |candidate: &FederationOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_federation_overrides(
    store: Rc<dyn FederationOverrideStore>,
    query: &FederationOverrideQuery,
) -> Vec<FederationOverride> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FederationOverrideStore> = new_memory_federation_override_store();
        let item = new_federation_override("code-913", "title-258");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
