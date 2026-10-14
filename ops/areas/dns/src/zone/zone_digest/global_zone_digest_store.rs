use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global zone digests by code.
pub trait GlobalZoneDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalZoneDigest>;
    fn save(&self, item: &GlobalZoneDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalZoneDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalZoneDigestStore {
    pub items: RefCell<BTreeMap<String, GlobalZoneDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalZoneDigestStore for MemoryGlobalZoneDigestStore {
    fn find(&self, code: &str) -> Option<GlobalZoneDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalZoneDigest) {
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

    fn list(&self) -> Vec<GlobalZoneDigest> {
        let mut out: Vec<GlobalZoneDigest> = Vec::new();
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

pub fn new_memory_global_zone_digest_store() -> Rc<dyn GlobalZoneDigestStore> {
    Rc::new(MemoryGlobalZoneDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalZoneDigestStore {
    pub inner: Rc<dyn GlobalZoneDigestStore>,
    pub cache: RefCell<cache::TtlCache<GlobalZoneDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalZoneDigestStore for CachedGlobalZoneDigestStore {
    fn find(&self, code: &str) -> Option<GlobalZoneDigest> {
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

    fn save(&self, item: &GlobalZoneDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalZoneDigest> {
        self.inner.list()
    }
}

pub fn new_cached_global_zone_digest_store(
    inner: Rc<dyn GlobalZoneDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalZoneDigestStore> {
    Rc::new(CachedGlobalZoneDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalZoneDigest>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global zone digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalZoneDigestQuery {
    pub channel: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalZoneDigestQuery {
    pub fn matches(&self, item: &GlobalZoneDigest) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
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

    pub fn apply(&self, items: &[GlobalZoneDigest]) -> Vec<GlobalZoneDigest> {
        let matched = collections::filter(items, |candidate: &GlobalZoneDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_zone_digests(
    store: Rc<dyn GlobalZoneDigestStore>,
    query: &GlobalZoneDigestQuery,
) -> Vec<GlobalZoneDigest> {
    query.apply(&store.list())
}
