use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending retention digests by code.
pub trait PendingRetentionDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingRetentionDigest>;
    fn save(&self, item: &PendingRetentionDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingRetentionDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingRetentionDigestStore {
    pub items: RefCell<BTreeMap<String, PendingRetentionDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingRetentionDigestStore for MemoryPendingRetentionDigestStore {
    fn find(&self, code: &str) -> Option<PendingRetentionDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingRetentionDigest) {
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

    fn list(&self) -> Vec<PendingRetentionDigest> {
        let mut out: Vec<PendingRetentionDigest> = Vec::new();
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

pub fn new_memory_pending_retention_digest_store() -> Rc<dyn PendingRetentionDigestStore> {
    Rc::new(MemoryPendingRetentionDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingRetentionDigestStore {
    pub inner: Rc<dyn PendingRetentionDigestStore>,
    pub cache: RefCell<cache::TtlCache<PendingRetentionDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingRetentionDigestStore for CachedPendingRetentionDigestStore {
    fn find(&self, code: &str) -> Option<PendingRetentionDigest> {
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

    fn save(&self, item: &PendingRetentionDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingRetentionDigest> {
        self.inner.list()
    }
}

pub fn new_cached_pending_retention_digest_store(
    inner: Rc<dyn PendingRetentionDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingRetentionDigestStore> {
    Rc::new(CachedPendingRetentionDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingRetentionDigest>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending retention digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionDigestQuery {
    pub title: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingRetentionDigestQuery {
    pub fn matches(&self, item: &PendingRetentionDigest) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PendingRetentionDigest]) -> Vec<PendingRetentionDigest> {
        let matched = collections::filter(items, |candidate: &PendingRetentionDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_retention_digests(
    store: Rc<dyn PendingRetentionDigestStore>,
    query: &PendingRetentionDigestQuery,
) -> Vec<PendingRetentionDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PendingRetentionDigestStore> = new_memory_pending_retention_digest_store();
        let item = new_pending_retention_digest("code-427", "title-762");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
