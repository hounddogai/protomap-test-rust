use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps retention claims by code.
pub trait RetentionClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RetentionClaim>;
    fn save(&self, item: &RetentionClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RetentionClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRetentionClaimStore {
    pub items: RefCell<BTreeMap<String, RetentionClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl RetentionClaimStore for MemoryRetentionClaimStore {
    fn find(&self, code: &str) -> Option<RetentionClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RetentionClaim) {
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

    fn list(&self) -> Vec<RetentionClaim> {
        let mut out: Vec<RetentionClaim> = Vec::new();
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

pub fn new_memory_retention_claim_store() -> Rc<dyn RetentionClaimStore> {
    Rc::new(MemoryRetentionClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRetentionClaimStore {
    pub inner: Rc<dyn RetentionClaimStore>,
    pub cache: RefCell<cache::TtlCache<RetentionClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RetentionClaimStore for CachedRetentionClaimStore {
    fn find(&self, code: &str) -> Option<RetentionClaim> {
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

    fn save(&self, item: &RetentionClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RetentionClaim> {
        self.inner.list()
    }
}

pub fn new_cached_retention_claim_store(
    inner: Rc<dyn RetentionClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RetentionClaimStore> {
    Rc::new(CachedRetentionClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RetentionClaim>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects retention claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionClaimQuery {
    pub title: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RetentionClaimQuery {
    pub fn matches(&self, item: &RetentionClaim) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RetentionClaim]) -> Vec<RetentionClaim> {
        let matched = collections::filter(items, |candidate: &RetentionClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_retention_claims(store: Rc<dyn RetentionClaimStore>, query: &RetentionClaimQuery) -> Vec<RetentionClaim> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RetentionClaimStore> = new_memory_retention_claim_store();
        let item = new_retention_claim("code-338", "title-994");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
