use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary nonce digests by code.
pub trait PrimaryNonceDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryNonceDigest>;
    fn save(&self, item: &PrimaryNonceDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryNonceDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryNonceDigestStore {
    pub items: RefCell<BTreeMap<String, PrimaryNonceDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryNonceDigestStore for MemoryPrimaryNonceDigestStore {
    fn find(&self, code: &str) -> Option<PrimaryNonceDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryNonceDigest) {
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

    fn list(&self) -> Vec<PrimaryNonceDigest> {
        let mut out: Vec<PrimaryNonceDigest> = Vec::new();
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

pub fn new_memory_primary_nonce_digest_store() -> Rc<dyn PrimaryNonceDigestStore> {
    Rc::new(MemoryPrimaryNonceDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryNonceDigestStore {
    pub inner: Rc<dyn PrimaryNonceDigestStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryNonceDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryNonceDigestStore for CachedPrimaryNonceDigestStore {
    fn find(&self, code: &str) -> Option<PrimaryNonceDigest> {
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

    fn save(&self, item: &PrimaryNonceDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryNonceDigest> {
        self.inner.list()
    }
}

pub fn new_cached_primary_nonce_digest_store(
    inner: Rc<dyn PrimaryNonceDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryNonceDigestStore> {
    Rc::new(CachedPrimaryNonceDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryNonceDigest>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary nonce digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryNonceDigestQuery {
    pub segment: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryNonceDigestQuery {
    pub fn matches(&self, item: &PrimaryNonceDigest) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryNonceDigest]) -> Vec<PrimaryNonceDigest> {
        let matched = collections::filter(items, |candidate: &PrimaryNonceDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_nonce_digests(
    store: Rc<dyn PrimaryNonceDigestStore>,
    query: &PrimaryNonceDigestQuery,
) -> Vec<PrimaryNonceDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryNonceDigestStore> = new_memory_primary_nonce_digest_store();
        let item = new_primary_nonce_digest("code-742", "name-251");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
