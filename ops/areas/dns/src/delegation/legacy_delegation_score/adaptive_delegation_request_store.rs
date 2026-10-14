use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive delegation requests by code.
pub trait AdaptiveDelegationRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveDelegationRequest>;
    fn save(&self, item: &AdaptiveDelegationRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveDelegationRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveDelegationRequestStore {
    pub items: RefCell<BTreeMap<String, AdaptiveDelegationRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveDelegationRequestStore for MemoryAdaptiveDelegationRequestStore {
    fn find(&self, code: &str) -> Option<AdaptiveDelegationRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveDelegationRequest) {
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

    fn list(&self) -> Vec<AdaptiveDelegationRequest> {
        let mut out: Vec<AdaptiveDelegationRequest> = Vec::new();
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

pub fn new_memory_adaptive_delegation_request_store() -> Rc<dyn AdaptiveDelegationRequestStore> {
    Rc::new(MemoryAdaptiveDelegationRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveDelegationRequestStore {
    pub inner: Rc<dyn AdaptiveDelegationRequestStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveDelegationRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveDelegationRequestStore for CachedAdaptiveDelegationRequestStore {
    fn find(&self, code: &str) -> Option<AdaptiveDelegationRequest> {
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

    fn save(&self, item: &AdaptiveDelegationRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveDelegationRequest> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_delegation_request_store(
    inner: Rc<dyn AdaptiveDelegationRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveDelegationRequestStore> {
    Rc::new(CachedAdaptiveDelegationRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveDelegationRequest>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive delegation requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationRequestQuery {
    pub name: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveDelegationRequestQuery {
    pub fn matches(&self, item: &AdaptiveDelegationRequest) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveDelegationRequest]) -> Vec<AdaptiveDelegationRequest> {
        let matched = collections::filter(items, |candidate: &AdaptiveDelegationRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_delegation_requests(
    store: Rc<dyn AdaptiveDelegationRequestStore>,
    query: &AdaptiveDelegationRequestQuery,
) -> Vec<AdaptiveDelegationRequest> {
    query.apply(&store.list())
}
