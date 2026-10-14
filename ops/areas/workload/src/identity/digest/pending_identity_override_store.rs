use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending identity overrides by code.
pub trait PendingIdentityOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingIdentityOverride>;
    fn save(&self, item: &PendingIdentityOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingIdentityOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingIdentityOverrideStore {
    pub items: RefCell<BTreeMap<String, PendingIdentityOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingIdentityOverrideStore for MemoryPendingIdentityOverrideStore {
    fn find(&self, code: &str) -> Option<PendingIdentityOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingIdentityOverride) {
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

    fn list(&self) -> Vec<PendingIdentityOverride> {
        let mut out: Vec<PendingIdentityOverride> = Vec::new();
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

pub fn new_memory_pending_identity_override_store() -> Rc<dyn PendingIdentityOverrideStore> {
    Rc::new(MemoryPendingIdentityOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingIdentityOverrideStore {
    pub inner: Rc<dyn PendingIdentityOverrideStore>,
    pub cache: RefCell<cache::TtlCache<PendingIdentityOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingIdentityOverrideStore for CachedPendingIdentityOverrideStore {
    fn find(&self, code: &str) -> Option<PendingIdentityOverride> {
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

    fn save(&self, item: &PendingIdentityOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingIdentityOverride> {
        self.inner.list()
    }
}

pub fn new_cached_pending_identity_override_store(
    inner: Rc<dyn PendingIdentityOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingIdentityOverrideStore> {
    Rc::new(CachedPendingIdentityOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingIdentityOverride>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending identity overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentityOverrideQuery {
    pub scope: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingIdentityOverrideQuery {
    pub fn matches(&self, item: &PendingIdentityOverride) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PendingIdentityOverride]) -> Vec<PendingIdentityOverride> {
        let matched = collections::filter(items, |candidate: &PendingIdentityOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_identity_overrides(
    store: Rc<dyn PendingIdentityOverrideStore>,
    query: &PendingIdentityOverrideQuery,
) -> Vec<PendingIdentityOverride> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PendingIdentityOverrideStore> = new_memory_pending_identity_override_store();
        let item = new_pending_identity_override("code-141", "name-46");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
