use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy delegation requests by code.
pub trait LegacyDelegationRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyDelegationRequest>;
    fn save(&self, item: &LegacyDelegationRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyDelegationRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyDelegationRequestStore {
    pub items: RefCell<BTreeMap<String, LegacyDelegationRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyDelegationRequestStore for MemoryLegacyDelegationRequestStore {
    fn find(&self, code: &str) -> Option<LegacyDelegationRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyDelegationRequest) {
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

    fn list(&self) -> Vec<LegacyDelegationRequest> {
        let mut out: Vec<LegacyDelegationRequest> = Vec::new();
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

pub fn new_memory_legacy_delegation_request_store() -> Rc<dyn LegacyDelegationRequestStore> {
    Rc::new(MemoryLegacyDelegationRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyDelegationRequestStore {
    pub inner: Rc<dyn LegacyDelegationRequestStore>,
    pub cache: RefCell<cache::TtlCache<LegacyDelegationRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyDelegationRequestStore for CachedLegacyDelegationRequestStore {
    fn find(&self, code: &str) -> Option<LegacyDelegationRequest> {
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

    fn save(&self, item: &LegacyDelegationRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyDelegationRequest> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_delegation_request_store(
    inner: Rc<dyn LegacyDelegationRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyDelegationRequestStore> {
    Rc::new(CachedLegacyDelegationRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyDelegationRequest>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy delegation requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDelegationRequestQuery {
    pub category: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyDelegationRequestQuery {
    pub fn matches(&self, item: &LegacyDelegationRequest) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyDelegationRequest]) -> Vec<LegacyDelegationRequest> {
        let matched = collections::filter(items, |candidate: &LegacyDelegationRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_delegation_requests(
    store: Rc<dyn LegacyDelegationRequestStore>,
    query: &LegacyDelegationRequestQuery,
) -> Vec<LegacyDelegationRequest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyDelegationRequestStore> = new_memory_legacy_delegation_request_store();
        let item = new_legacy_delegation_request("code-81", "title-895");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
