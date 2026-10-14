use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary failover digests by code.
pub trait PrimaryFailoverDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryFailoverDigest>;
    fn save(&self, item: &PrimaryFailoverDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryFailoverDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryFailoverDigestStore {
    pub items: RefCell<BTreeMap<String, PrimaryFailoverDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryFailoverDigestStore for MemoryPrimaryFailoverDigestStore {
    fn find(&self, code: &str) -> Option<PrimaryFailoverDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryFailoverDigest) {
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

    fn list(&self) -> Vec<PrimaryFailoverDigest> {
        let mut out: Vec<PrimaryFailoverDigest> = Vec::new();
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

pub fn new_memory_primary_failover_digest_store() -> Rc<dyn PrimaryFailoverDigestStore> {
    Rc::new(MemoryPrimaryFailoverDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryFailoverDigestStore {
    pub inner: Rc<dyn PrimaryFailoverDigestStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryFailoverDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryFailoverDigestStore for CachedPrimaryFailoverDigestStore {
    fn find(&self, code: &str) -> Option<PrimaryFailoverDigest> {
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

    fn save(&self, item: &PrimaryFailoverDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryFailoverDigest> {
        self.inner.list()
    }
}

pub fn new_cached_primary_failover_digest_store(
    inner: Rc<dyn PrimaryFailoverDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryFailoverDigestStore> {
    Rc::new(CachedPrimaryFailoverDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryFailoverDigest>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary failover digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverDigestQuery {
    pub title: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryFailoverDigestQuery {
    pub fn matches(&self, item: &PrimaryFailoverDigest) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryFailoverDigest]) -> Vec<PrimaryFailoverDigest> {
        let matched = collections::filter(items, |candidate: &PrimaryFailoverDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_failover_digests(
    store: Rc<dyn PrimaryFailoverDigestStore>,
    query: &PrimaryFailoverDigestQuery,
) -> Vec<PrimaryFailoverDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryFailoverDigestStore> = new_memory_primary_failover_digest_store();
        let item = new_primary_failover_digest("code-623", "title-762");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
