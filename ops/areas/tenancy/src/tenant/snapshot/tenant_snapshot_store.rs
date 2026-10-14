use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tenant snapshots by code.
pub trait TenantSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TenantSnapshot>;
    fn save(&self, item: &TenantSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TenantSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTenantSnapshotStore {
    pub items: RefCell<BTreeMap<String, TenantSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl TenantSnapshotStore for MemoryTenantSnapshotStore {
    fn find(&self, code: &str) -> Option<TenantSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TenantSnapshot) {
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

    fn list(&self) -> Vec<TenantSnapshot> {
        let mut out: Vec<TenantSnapshot> = Vec::new();
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

pub fn new_memory_tenant_snapshot_store() -> Rc<dyn TenantSnapshotStore> {
    Rc::new(MemoryTenantSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTenantSnapshotStore {
    pub inner: Rc<dyn TenantSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<TenantSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TenantSnapshotStore for CachedTenantSnapshotStore {
    fn find(&self, code: &str) -> Option<TenantSnapshot> {
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

    fn save(&self, item: &TenantSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TenantSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_tenant_snapshot_store(
    inner: Rc<dyn TenantSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TenantSnapshotStore> {
    Rc::new(CachedTenantSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TenantSnapshot>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tenant snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantSnapshotQuery {
    pub source: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TenantSnapshotQuery {
    pub fn matches(&self, item: &TenantSnapshot) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TenantSnapshot]) -> Vec<TenantSnapshot> {
        let matched = collections::filter(items, |candidate: &TenantSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tenant_snapshots(store: Rc<dyn TenantSnapshotStore>, query: &TenantSnapshotQuery) -> Vec<TenantSnapshot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TenantSnapshotStore> = new_memory_tenant_snapshot_store();
        let item = new_tenant_snapshot("code-850", "name-442");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
