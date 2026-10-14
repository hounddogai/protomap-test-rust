use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched token manifests by code.
pub trait BatchedTokenManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedTokenManifest>;
    fn save(&self, item: &BatchedTokenManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedTokenManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedTokenManifestStore {
    pub items: RefCell<BTreeMap<String, BatchedTokenManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedTokenManifestStore for MemoryBatchedTokenManifestStore {
    fn find(&self, code: &str) -> Option<BatchedTokenManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedTokenManifest) {
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

    fn list(&self) -> Vec<BatchedTokenManifest> {
        let mut out: Vec<BatchedTokenManifest> = Vec::new();
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

pub fn new_memory_batched_token_manifest_store() -> Rc<dyn BatchedTokenManifestStore> {
    Rc::new(MemoryBatchedTokenManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedTokenManifestStore {
    pub inner: Rc<dyn BatchedTokenManifestStore>,
    pub cache: RefCell<cache::TtlCache<BatchedTokenManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedTokenManifestStore for CachedBatchedTokenManifestStore {
    fn find(&self, code: &str) -> Option<BatchedTokenManifest> {
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

    fn save(&self, item: &BatchedTokenManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedTokenManifest> {
        self.inner.list()
    }
}

pub fn new_cached_batched_token_manifest_store(
    inner: Rc<dyn BatchedTokenManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedTokenManifestStore> {
    Rc::new(CachedBatchedTokenManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedTokenManifest>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched token manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenManifestQuery {
    pub locale: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedTokenManifestQuery {
    pub fn matches(&self, item: &BatchedTokenManifest) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedTokenManifest]) -> Vec<BatchedTokenManifest> {
        let matched = collections::filter(items, |candidate: &BatchedTokenManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_token_manifests(
    store: Rc<dyn BatchedTokenManifestStore>,
    query: &BatchedTokenManifestQuery,
) -> Vec<BatchedTokenManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedTokenManifestStore> = new_memory_batched_token_manifest_store();
        let item = new_batched_token_manifest("code-277", "name-223");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
