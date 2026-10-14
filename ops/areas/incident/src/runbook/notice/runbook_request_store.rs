use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps runbook requests by code.
pub trait RunbookRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RunbookRequest>;
    fn save(&self, item: &RunbookRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RunbookRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRunbookRequestStore {
    pub items: RefCell<BTreeMap<String, RunbookRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl RunbookRequestStore for MemoryRunbookRequestStore {
    fn find(&self, code: &str) -> Option<RunbookRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RunbookRequest) {
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

    fn list(&self) -> Vec<RunbookRequest> {
        let mut out: Vec<RunbookRequest> = Vec::new();
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

pub fn new_memory_runbook_request_store() -> Rc<dyn RunbookRequestStore> {
    Rc::new(MemoryRunbookRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRunbookRequestStore {
    pub inner: Rc<dyn RunbookRequestStore>,
    pub cache: RefCell<cache::TtlCache<RunbookRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RunbookRequestStore for CachedRunbookRequestStore {
    fn find(&self, code: &str) -> Option<RunbookRequest> {
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

    fn save(&self, item: &RunbookRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RunbookRequest> {
        self.inner.list()
    }
}

pub fn new_cached_runbook_request_store(
    inner: Rc<dyn RunbookRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RunbookRequestStore> {
    Rc::new(CachedRunbookRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RunbookRequest>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects runbook requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookRequestQuery {
    pub title: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RunbookRequestQuery {
    pub fn matches(&self, item: &RunbookRequest) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RunbookRequest]) -> Vec<RunbookRequest> {
        let matched = collections::filter(items, |candidate: &RunbookRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_runbook_requests(store: Rc<dyn RunbookRequestStore>, query: &RunbookRequestQuery) -> Vec<RunbookRequest> {
    query.apply(&store.list())
}
