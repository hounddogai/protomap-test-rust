use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered ttl policy snapshots by code.
pub trait TieredTtlPolicySnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredTtlPolicySnapshot>;
    fn save(&self, item: &TieredTtlPolicySnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredTtlPolicySnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredTtlPolicySnapshotStore {
    pub items: RefCell<BTreeMap<String, TieredTtlPolicySnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredTtlPolicySnapshotStore for MemoryTieredTtlPolicySnapshotStore {
    fn find(&self, code: &str) -> Option<TieredTtlPolicySnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredTtlPolicySnapshot) {
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

    fn list(&self) -> Vec<TieredTtlPolicySnapshot> {
        let mut out: Vec<TieredTtlPolicySnapshot> = Vec::new();
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

pub fn new_memory_tiered_ttl_policy_snapshot_store() -> Rc<dyn TieredTtlPolicySnapshotStore> {
    Rc::new(MemoryTieredTtlPolicySnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredTtlPolicySnapshotStore {
    pub inner: Rc<dyn TieredTtlPolicySnapshotStore>,
    pub cache: RefCell<cache::TtlCache<TieredTtlPolicySnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredTtlPolicySnapshotStore for CachedTieredTtlPolicySnapshotStore {
    fn find(&self, code: &str) -> Option<TieredTtlPolicySnapshot> {
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

    fn save(&self, item: &TieredTtlPolicySnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredTtlPolicySnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_ttl_policy_snapshot_store(
    inner: Rc<dyn TieredTtlPolicySnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredTtlPolicySnapshotStore> {
    Rc::new(CachedTieredTtlPolicySnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredTtlPolicySnapshot>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered ttl policy snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicySnapshotQuery {
    pub title: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredTtlPolicySnapshotQuery {
    pub fn matches(&self, item: &TieredTtlPolicySnapshot) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredTtlPolicySnapshot]) -> Vec<TieredTtlPolicySnapshot> {
        let matched = collections::filter(items, |candidate: &TieredTtlPolicySnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_ttl_policy_snapshots(
    store: Rc<dyn TieredTtlPolicySnapshotStore>,
    query: &TieredTtlPolicySnapshotQuery,
) -> Vec<TieredTtlPolicySnapshot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredTtlPolicySnapshotStore> = new_memory_tiered_ttl_policy_snapshot_store();
        let item = new_tiered_ttl_policy_snapshot("code-332", "title-87");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
