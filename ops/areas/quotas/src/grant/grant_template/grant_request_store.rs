use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps grant requests by code.
pub trait GrantRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GrantRequest>;
    fn save(&self, item: &GrantRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GrantRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGrantRequestStore {
    pub items: RefCell<BTreeMap<String, GrantRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl GrantRequestStore for MemoryGrantRequestStore {
    fn find(&self, code: &str) -> Option<GrantRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GrantRequest) {
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

    fn list(&self) -> Vec<GrantRequest> {
        let mut out: Vec<GrantRequest> = Vec::new();
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

pub fn new_memory_grant_request_store() -> Rc<dyn GrantRequestStore> {
    Rc::new(MemoryGrantRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGrantRequestStore {
    pub inner: Rc<dyn GrantRequestStore>,
    pub cache: RefCell<cache::TtlCache<GrantRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GrantRequestStore for CachedGrantRequestStore {
    fn find(&self, code: &str) -> Option<GrantRequest> {
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

    fn save(&self, item: &GrantRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GrantRequest> {
        self.inner.list()
    }
}

pub fn new_cached_grant_request_store(
    inner: Rc<dyn GrantRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GrantRequestStore> {
    Rc::new(CachedGrantRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GrantRequest>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects grant requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantRequestQuery {
    pub country: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GrantRequestQuery {
    pub fn matches(&self, item: &GrantRequest) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GrantRequest]) -> Vec<GrantRequest> {
        let matched = collections::filter(items, |candidate: &GrantRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_grant_requests(store: Rc<dyn GrantRequestStore>, query: &GrantRequestQuery) -> Vec<GrantRequest> {
    query.apply(&store.list())
}
