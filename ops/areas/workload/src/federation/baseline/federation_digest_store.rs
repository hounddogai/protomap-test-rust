use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps federation digests by code.
pub trait FederationDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FederationDigest>;
    fn save(&self, item: &FederationDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FederationDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFederationDigestStore {
    pub items: RefCell<BTreeMap<String, FederationDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl FederationDigestStore for MemoryFederationDigestStore {
    fn find(&self, code: &str) -> Option<FederationDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FederationDigest) {
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

    fn list(&self) -> Vec<FederationDigest> {
        let mut out: Vec<FederationDigest> = Vec::new();
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

pub fn new_memory_federation_digest_store() -> Rc<dyn FederationDigestStore> {
    Rc::new(MemoryFederationDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFederationDigestStore {
    pub inner: Rc<dyn FederationDigestStore>,
    pub cache: RefCell<cache::TtlCache<FederationDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FederationDigestStore for CachedFederationDigestStore {
    fn find(&self, code: &str) -> Option<FederationDigest> {
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

    fn save(&self, item: &FederationDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FederationDigest> {
        self.inner.list()
    }
}

pub fn new_cached_federation_digest_store(
    inner: Rc<dyn FederationDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FederationDigestStore> {
    Rc::new(CachedFederationDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FederationDigest>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects federation digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationDigestQuery {
    pub label: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FederationDigestQuery {
    pub fn matches(&self, item: &FederationDigest) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[FederationDigest]) -> Vec<FederationDigest> {
        let matched = collections::filter(items, |candidate: &FederationDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_federation_digests(
    store: Rc<dyn FederationDigestStore>,
    query: &FederationDigestQuery,
) -> Vec<FederationDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FederationDigestStore> = new_memory_federation_digest_store();
        let item = new_federation_digest("code-653", "label-437");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
