use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps release digests by code.
pub trait ReleaseDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ReleaseDigest>;
    fn save(&self, item: &ReleaseDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ReleaseDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryReleaseDigestStore {
    pub items: RefCell<BTreeMap<String, ReleaseDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl ReleaseDigestStore for MemoryReleaseDigestStore {
    fn find(&self, code: &str) -> Option<ReleaseDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ReleaseDigest) {
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

    fn list(&self) -> Vec<ReleaseDigest> {
        let mut out: Vec<ReleaseDigest> = Vec::new();
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

pub fn new_memory_release_digest_store() -> Rc<dyn ReleaseDigestStore> {
    Rc::new(MemoryReleaseDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedReleaseDigestStore {
    pub inner: Rc<dyn ReleaseDigestStore>,
    pub cache: RefCell<cache::TtlCache<ReleaseDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ReleaseDigestStore for CachedReleaseDigestStore {
    fn find(&self, code: &str) -> Option<ReleaseDigest> {
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

    fn save(&self, item: &ReleaseDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ReleaseDigest> {
        self.inner.list()
    }
}

pub fn new_cached_release_digest_store(
    inner: Rc<dyn ReleaseDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ReleaseDigestStore> {
    Rc::new(CachedReleaseDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ReleaseDigest>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects release digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseDigestQuery {
    pub label: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ReleaseDigestQuery {
    pub fn matches(&self, item: &ReleaseDigest) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.cost < self.min_cost {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ReleaseDigest]) -> Vec<ReleaseDigest> {
        let matched = collections::filter(items, |candidate: &ReleaseDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_release_digests(store: Rc<dyn ReleaseDigestStore>, query: &ReleaseDigestQuery) -> Vec<ReleaseDigest> {
    query.apply(&store.list())
}
