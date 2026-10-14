use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps zone snapshots by code.
pub trait ZoneSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ZoneSnapshot>;
    fn save(&self, item: &ZoneSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ZoneSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryZoneSnapshotStore {
    pub items: RefCell<BTreeMap<String, ZoneSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl ZoneSnapshotStore for MemoryZoneSnapshotStore {
    fn find(&self, code: &str) -> Option<ZoneSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ZoneSnapshot) {
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

    fn list(&self) -> Vec<ZoneSnapshot> {
        let mut out: Vec<ZoneSnapshot> = Vec::new();
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

pub fn new_memory_zone_snapshot_store() -> Rc<dyn ZoneSnapshotStore> {
    Rc::new(MemoryZoneSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedZoneSnapshotStore {
    pub inner: Rc<dyn ZoneSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<ZoneSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ZoneSnapshotStore for CachedZoneSnapshotStore {
    fn find(&self, code: &str) -> Option<ZoneSnapshot> {
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

    fn save(&self, item: &ZoneSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ZoneSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_zone_snapshot_store(
    inner: Rc<dyn ZoneSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ZoneSnapshotStore> {
    Rc::new(CachedZoneSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ZoneSnapshot>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects zone snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSnapshotQuery {
    pub namespace: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ZoneSnapshotQuery {
    pub fn matches(&self, item: &ZoneSnapshot) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ZoneSnapshot]) -> Vec<ZoneSnapshot> {
        let matched = collections::filter(items, |candidate: &ZoneSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_zone_snapshots(store: Rc<dyn ZoneSnapshotStore>, query: &ZoneSnapshotQuery) -> Vec<ZoneSnapshot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ZoneSnapshotStore> = new_memory_zone_snapshot_store();
        let item = new_zone_snapshot("code-677", "name-941");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
