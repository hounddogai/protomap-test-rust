use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps probe digests by code.
pub trait ProbeDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ProbeDigest>;
    fn save(&self, item: &ProbeDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ProbeDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryProbeDigestStore {
    pub items: RefCell<BTreeMap<String, ProbeDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl ProbeDigestStore for MemoryProbeDigestStore {
    fn find(&self, code: &str) -> Option<ProbeDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ProbeDigest) {
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

    fn list(&self) -> Vec<ProbeDigest> {
        let mut out: Vec<ProbeDigest> = Vec::new();
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

pub fn new_memory_probe_digest_store() -> Rc<dyn ProbeDigestStore> {
    Rc::new(MemoryProbeDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedProbeDigestStore {
    pub inner: Rc<dyn ProbeDigestStore>,
    pub cache: RefCell<cache::TtlCache<ProbeDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ProbeDigestStore for CachedProbeDigestStore {
    fn find(&self, code: &str) -> Option<ProbeDigest> {
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

    fn save(&self, item: &ProbeDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ProbeDigest> {
        self.inner.list()
    }
}

pub fn new_cached_probe_digest_store(
    inner: Rc<dyn ProbeDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ProbeDigestStore> {
    Rc::new(CachedProbeDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ProbeDigest>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects probe digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeDigestQuery {
    pub category: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ProbeDigestQuery {
    pub fn matches(&self, item: &ProbeDigest) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ProbeDigest]) -> Vec<ProbeDigest> {
        let matched = collections::filter(items, |candidate: &ProbeDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_probe_digests(store: Rc<dyn ProbeDigestStore>, query: &ProbeDigestQuery) -> Vec<ProbeDigest> {
    query.apply(&store.list())
}
