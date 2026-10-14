use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global isolation claims by code.
pub trait GlobalIsolationClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalIsolationClaim>;
    fn save(&self, item: &GlobalIsolationClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalIsolationClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalIsolationClaimStore {
    pub items: RefCell<BTreeMap<String, GlobalIsolationClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalIsolationClaimStore for MemoryGlobalIsolationClaimStore {
    fn find(&self, code: &str) -> Option<GlobalIsolationClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalIsolationClaim) {
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

    fn list(&self) -> Vec<GlobalIsolationClaim> {
        let mut out: Vec<GlobalIsolationClaim> = Vec::new();
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

pub fn new_memory_global_isolation_claim_store() -> Rc<dyn GlobalIsolationClaimStore> {
    Rc::new(MemoryGlobalIsolationClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalIsolationClaimStore {
    pub inner: Rc<dyn GlobalIsolationClaimStore>,
    pub cache: RefCell<cache::TtlCache<GlobalIsolationClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalIsolationClaimStore for CachedGlobalIsolationClaimStore {
    fn find(&self, code: &str) -> Option<GlobalIsolationClaim> {
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

    fn save(&self, item: &GlobalIsolationClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalIsolationClaim> {
        self.inner.list()
    }
}

pub fn new_cached_global_isolation_claim_store(
    inner: Rc<dyn GlobalIsolationClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalIsolationClaimStore> {
    Rc::new(CachedGlobalIsolationClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalIsolationClaim>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global isolation claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIsolationClaimQuery {
    pub name: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalIsolationClaimQuery {
    pub fn matches(&self, item: &GlobalIsolationClaim) -> bool {
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

    pub fn apply(&self, items: &[GlobalIsolationClaim]) -> Vec<GlobalIsolationClaim> {
        let matched = collections::filter(items, |candidate: &GlobalIsolationClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_isolation_claims(
    store: Rc<dyn GlobalIsolationClaimStore>,
    query: &GlobalIsolationClaimQuery,
) -> Vec<GlobalIsolationClaim> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalIsolationClaimStore> = new_memory_global_isolation_claim_store();
        let item = new_global_isolation_claim("code-220", "name-729");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
