use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps retention snapshots by code.
pub trait RetentionSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RetentionSnapshot>;
    fn save(&self, item: &RetentionSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RetentionSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRetentionSnapshotStore {
    pub items: RefCell<BTreeMap<String, RetentionSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl RetentionSnapshotStore for MemoryRetentionSnapshotStore {
    fn find(&self, code: &str) -> Option<RetentionSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RetentionSnapshot) {
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

    fn list(&self) -> Vec<RetentionSnapshot> {
        let mut out: Vec<RetentionSnapshot> = Vec::new();
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

pub fn new_memory_retention_snapshot_store() -> Rc<dyn RetentionSnapshotStore> {
    Rc::new(MemoryRetentionSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRetentionSnapshotStore {
    pub inner: Rc<dyn RetentionSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<RetentionSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RetentionSnapshotStore for CachedRetentionSnapshotStore {
    fn find(&self, code: &str) -> Option<RetentionSnapshot> {
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

    fn save(&self, item: &RetentionSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RetentionSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_retention_snapshot_store(
    inner: Rc<dyn RetentionSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RetentionSnapshotStore> {
    Rc::new(CachedRetentionSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RetentionSnapshot>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects retention snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSnapshotQuery {
    pub name: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RetentionSnapshotQuery {
    pub fn matches(&self, item: &RetentionSnapshot) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RetentionSnapshot]) -> Vec<RetentionSnapshot> {
        let matched = collections::filter(items, |candidate: &RetentionSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_retention_snapshots(
    store: Rc<dyn RetentionSnapshotStore>,
    query: &RetentionSnapshotQuery,
) -> Vec<RetentionSnapshot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RetentionSnapshotStore> = new_memory_retention_snapshot_store();
        let item = new_retention_snapshot("code-767", "name-9");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
