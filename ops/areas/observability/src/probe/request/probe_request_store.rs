use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps probe requests by code.
pub trait ProbeRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ProbeRequest>;
    fn save(&self, item: &ProbeRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ProbeRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryProbeRequestStore {
    pub items: RefCell<BTreeMap<String, ProbeRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl ProbeRequestStore for MemoryProbeRequestStore {
    fn find(&self, code: &str) -> Option<ProbeRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ProbeRequest) {
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

    fn list(&self) -> Vec<ProbeRequest> {
        let mut out: Vec<ProbeRequest> = Vec::new();
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

pub fn new_memory_probe_request_store() -> Rc<dyn ProbeRequestStore> {
    Rc::new(MemoryProbeRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedProbeRequestStore {
    pub inner: Rc<dyn ProbeRequestStore>,
    pub cache: RefCell<cache::TtlCache<ProbeRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ProbeRequestStore for CachedProbeRequestStore {
    fn find(&self, code: &str) -> Option<ProbeRequest> {
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

    fn save(&self, item: &ProbeRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ProbeRequest> {
        self.inner.list()
    }
}

pub fn new_cached_probe_request_store(
    inner: Rc<dyn ProbeRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ProbeRequestStore> {
    Rc::new(CachedProbeRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ProbeRequest>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects probe requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeRequestQuery {
    pub cluster: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ProbeRequestQuery {
    pub fn matches(&self, item: &ProbeRequest) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ProbeRequest]) -> Vec<ProbeRequest> {
        let matched = collections::filter(items, |candidate: &ProbeRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_probe_requests(store: Rc<dyn ProbeRequestStore>, query: &ProbeRequestQuery) -> Vec<ProbeRequest> {
    query.apply(&store.list())
}
