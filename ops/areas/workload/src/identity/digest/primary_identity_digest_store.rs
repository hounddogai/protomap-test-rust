use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary identity digests by code.
pub trait PrimaryIdentityDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryIdentityDigest>;
    fn save(&self, item: &PrimaryIdentityDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryIdentityDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryIdentityDigestStore {
    pub items: RefCell<BTreeMap<String, PrimaryIdentityDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryIdentityDigestStore for MemoryPrimaryIdentityDigestStore {
    fn find(&self, code: &str) -> Option<PrimaryIdentityDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryIdentityDigest) {
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

    fn list(&self) -> Vec<PrimaryIdentityDigest> {
        let mut out: Vec<PrimaryIdentityDigest> = Vec::new();
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

pub fn new_memory_primary_identity_digest_store() -> Rc<dyn PrimaryIdentityDigestStore> {
    Rc::new(MemoryPrimaryIdentityDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryIdentityDigestStore {
    pub inner: Rc<dyn PrimaryIdentityDigestStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryIdentityDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryIdentityDigestStore for CachedPrimaryIdentityDigestStore {
    fn find(&self, code: &str) -> Option<PrimaryIdentityDigest> {
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

    fn save(&self, item: &PrimaryIdentityDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryIdentityDigest> {
        self.inner.list()
    }
}

pub fn new_cached_primary_identity_digest_store(
    inner: Rc<dyn PrimaryIdentityDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryIdentityDigestStore> {
    Rc::new(CachedPrimaryIdentityDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryIdentityDigest>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary identity digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityDigestQuery {
    pub title: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryIdentityDigestQuery {
    pub fn matches(&self, item: &PrimaryIdentityDigest) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryIdentityDigest]) -> Vec<PrimaryIdentityDigest> {
        let matched = collections::filter(items, |candidate: &PrimaryIdentityDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_identity_digests(
    store: Rc<dyn PrimaryIdentityDigestStore>,
    query: &PrimaryIdentityDigestQuery,
) -> Vec<PrimaryIdentityDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryIdentityDigestStore> = new_memory_primary_identity_digest_store();
        let item = new_primary_identity_digest("code-551", "title-751");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
