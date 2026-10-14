use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling limit claims by code.
pub trait RollingLimitClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingLimitClaim>;
    fn save(&self, item: &RollingLimitClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingLimitClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingLimitClaimStore {
    pub items: RefCell<BTreeMap<String, RollingLimitClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingLimitClaimStore for MemoryRollingLimitClaimStore {
    fn find(&self, code: &str) -> Option<RollingLimitClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingLimitClaim) {
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

    fn list(&self) -> Vec<RollingLimitClaim> {
        let mut out: Vec<RollingLimitClaim> = Vec::new();
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

pub fn new_memory_rolling_limit_claim_store() -> Rc<dyn RollingLimitClaimStore> {
    Rc::new(MemoryRollingLimitClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingLimitClaimStore {
    pub inner: Rc<dyn RollingLimitClaimStore>,
    pub cache: RefCell<cache::TtlCache<RollingLimitClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingLimitClaimStore for CachedRollingLimitClaimStore {
    fn find(&self, code: &str) -> Option<RollingLimitClaim> {
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

    fn save(&self, item: &RollingLimitClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingLimitClaim> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_limit_claim_store(
    inner: Rc<dyn RollingLimitClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingLimitClaimStore> {
    Rc::new(CachedRollingLimitClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingLimitClaim>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling limit claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitClaimQuery {
    pub name: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingLimitClaimQuery {
    pub fn matches(&self, item: &RollingLimitClaim) -> bool {
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

    pub fn apply(&self, items: &[RollingLimitClaim]) -> Vec<RollingLimitClaim> {
        let matched = collections::filter(items, |candidate: &RollingLimitClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_limit_claims(
    store: Rc<dyn RollingLimitClaimStore>,
    query: &RollingLimitClaimQuery,
) -> Vec<RollingLimitClaim> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingLimitClaimStore> = new_memory_rolling_limit_claim_store();
        let item = new_rolling_limit_claim("code-848", "name-240");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
