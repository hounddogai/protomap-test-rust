use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps federation checkpoints by code.
pub trait FederationCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FederationCheckpoint>;
    fn save(&self, item: &FederationCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FederationCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFederationCheckpointStore {
    pub items: RefCell<BTreeMap<String, FederationCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl FederationCheckpointStore for MemoryFederationCheckpointStore {
    fn find(&self, code: &str) -> Option<FederationCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FederationCheckpoint) {
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

    fn list(&self) -> Vec<FederationCheckpoint> {
        let mut out: Vec<FederationCheckpoint> = Vec::new();
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

pub fn new_memory_federation_checkpoint_store() -> Rc<dyn FederationCheckpointStore> {
    Rc::new(MemoryFederationCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFederationCheckpointStore {
    pub inner: Rc<dyn FederationCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<FederationCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FederationCheckpointStore for CachedFederationCheckpointStore {
    fn find(&self, code: &str) -> Option<FederationCheckpoint> {
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

    fn save(&self, item: &FederationCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FederationCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_federation_checkpoint_store(
    inner: Rc<dyn FederationCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FederationCheckpointStore> {
    Rc::new(CachedFederationCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FederationCheckpoint>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects federation checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationCheckpointQuery {
    pub name: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FederationCheckpointQuery {
    pub fn matches(&self, item: &FederationCheckpoint) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FederationCheckpoint]) -> Vec<FederationCheckpoint> {
        let matched = collections::filter(items, |candidate: &FederationCheckpoint| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_federation_checkpoints(
    store: Rc<dyn FederationCheckpointStore>,
    query: &FederationCheckpointQuery,
) -> Vec<FederationCheckpoint> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FederationCheckpointStore> = new_memory_federation_checkpoint_store();
        let item = new_federation_checkpoint("code-976", "name-89");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
