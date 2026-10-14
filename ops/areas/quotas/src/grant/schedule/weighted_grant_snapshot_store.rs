use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted grant snapshots by code.
pub trait WeightedGrantSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedGrantSnapshot>;
    fn save(&self, item: &WeightedGrantSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedGrantSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedGrantSnapshotStore {
    pub items: RefCell<BTreeMap<String, WeightedGrantSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedGrantSnapshotStore for MemoryWeightedGrantSnapshotStore {
    fn find(&self, code: &str) -> Option<WeightedGrantSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedGrantSnapshot) {
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

    fn list(&self) -> Vec<WeightedGrantSnapshot> {
        let mut out: Vec<WeightedGrantSnapshot> = Vec::new();
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

pub fn new_memory_weighted_grant_snapshot_store() -> Rc<dyn WeightedGrantSnapshotStore> {
    Rc::new(MemoryWeightedGrantSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedGrantSnapshotStore {
    pub inner: Rc<dyn WeightedGrantSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<WeightedGrantSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedGrantSnapshotStore for CachedWeightedGrantSnapshotStore {
    fn find(&self, code: &str) -> Option<WeightedGrantSnapshot> {
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

    fn save(&self, item: &WeightedGrantSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedGrantSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_grant_snapshot_store(
    inner: Rc<dyn WeightedGrantSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedGrantSnapshotStore> {
    Rc::new(CachedWeightedGrantSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedGrantSnapshot>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted grant snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedGrantSnapshotQuery {
    pub segment: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedGrantSnapshotQuery {
    pub fn matches(&self, item: &WeightedGrantSnapshot) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.cost < self.min_cost {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedGrantSnapshot]) -> Vec<WeightedGrantSnapshot> {
        let matched = collections::filter(items, |candidate: &WeightedGrantSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_grant_snapshots(
    store: Rc<dyn WeightedGrantSnapshotStore>,
    query: &WeightedGrantSnapshotQuery,
) -> Vec<WeightedGrantSnapshot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedGrantSnapshotStore> = new_memory_weighted_grant_snapshot_store();
        let item = new_weighted_grant_snapshot("code-218", "label-705");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
