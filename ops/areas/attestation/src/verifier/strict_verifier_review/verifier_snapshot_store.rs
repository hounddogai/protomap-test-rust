use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps verifier snapshots by code.
pub trait VerifierSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<VerifierSnapshot>;
    fn save(&self, item: &VerifierSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<VerifierSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryVerifierSnapshotStore {
    pub items: RefCell<BTreeMap<String, VerifierSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl VerifierSnapshotStore for MemoryVerifierSnapshotStore {
    fn find(&self, code: &str) -> Option<VerifierSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &VerifierSnapshot) {
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

    fn list(&self) -> Vec<VerifierSnapshot> {
        let mut out: Vec<VerifierSnapshot> = Vec::new();
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

pub fn new_memory_verifier_snapshot_store() -> Rc<dyn VerifierSnapshotStore> {
    Rc::new(MemoryVerifierSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedVerifierSnapshotStore {
    pub inner: Rc<dyn VerifierSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<VerifierSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl VerifierSnapshotStore for CachedVerifierSnapshotStore {
    fn find(&self, code: &str) -> Option<VerifierSnapshot> {
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

    fn save(&self, item: &VerifierSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<VerifierSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_verifier_snapshot_store(
    inner: Rc<dyn VerifierSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn VerifierSnapshotStore> {
    Rc::new(CachedVerifierSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<VerifierSnapshot>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects verifier snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSnapshotQuery {
    pub cluster: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl VerifierSnapshotQuery {
    pub fn matches(&self, item: &VerifierSnapshot) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[VerifierSnapshot]) -> Vec<VerifierSnapshot> {
        let matched = collections::filter(items, |candidate: &VerifierSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_verifier_snapshots(
    store: Rc<dyn VerifierSnapshotStore>,
    query: &VerifierSnapshotQuery,
) -> Vec<VerifierSnapshot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn VerifierSnapshotStore> = new_memory_verifier_snapshot_store();
        let item = new_verifier_snapshot("code-934", "name-953");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
