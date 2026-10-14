use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps freeze requests by code.
pub trait FreezeRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FreezeRequest>;
    fn save(&self, item: &FreezeRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FreezeRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFreezeRequestStore {
    pub items: RefCell<BTreeMap<String, FreezeRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl FreezeRequestStore for MemoryFreezeRequestStore {
    fn find(&self, code: &str) -> Option<FreezeRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FreezeRequest) {
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

    fn list(&self) -> Vec<FreezeRequest> {
        let mut out: Vec<FreezeRequest> = Vec::new();
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

pub fn new_memory_freeze_request_store() -> Rc<dyn FreezeRequestStore> {
    Rc::new(MemoryFreezeRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFreezeRequestStore {
    pub inner: Rc<dyn FreezeRequestStore>,
    pub cache: RefCell<cache::TtlCache<FreezeRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FreezeRequestStore for CachedFreezeRequestStore {
    fn find(&self, code: &str) -> Option<FreezeRequest> {
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

    fn save(&self, item: &FreezeRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FreezeRequest> {
        self.inner.list()
    }
}

pub fn new_cached_freeze_request_store(
    inner: Rc<dyn FreezeRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FreezeRequestStore> {
    Rc::new(CachedFreezeRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FreezeRequest>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects freeze requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRequestQuery {
    pub region: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FreezeRequestQuery {
    pub fn matches(&self, item: &FreezeRequest) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[FreezeRequest]) -> Vec<FreezeRequest> {
        let matched = collections::filter(items, |candidate: &FreezeRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_freeze_requests(store: Rc<dyn FreezeRequestStore>, query: &FreezeRequestQuery) -> Vec<FreezeRequest> {
    query.apply(&store.list())
}
