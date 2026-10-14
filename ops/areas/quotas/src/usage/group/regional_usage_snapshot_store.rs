use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional usage snapshots by code.
pub trait RegionalUsageSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalUsageSnapshot>;
    fn save(&self, item: &RegionalUsageSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalUsageSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalUsageSnapshotStore {
    pub items: RefCell<BTreeMap<String, RegionalUsageSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalUsageSnapshotStore for MemoryRegionalUsageSnapshotStore {
    fn find(&self, code: &str) -> Option<RegionalUsageSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalUsageSnapshot) {
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

    fn list(&self) -> Vec<RegionalUsageSnapshot> {
        let mut out: Vec<RegionalUsageSnapshot> = Vec::new();
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

pub fn new_memory_regional_usage_snapshot_store() -> Rc<dyn RegionalUsageSnapshotStore> {
    Rc::new(MemoryRegionalUsageSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalUsageSnapshotStore {
    pub inner: Rc<dyn RegionalUsageSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<RegionalUsageSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalUsageSnapshotStore for CachedRegionalUsageSnapshotStore {
    fn find(&self, code: &str) -> Option<RegionalUsageSnapshot> {
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

    fn save(&self, item: &RegionalUsageSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalUsageSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_regional_usage_snapshot_store(
    inner: Rc<dyn RegionalUsageSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalUsageSnapshotStore> {
    Rc::new(CachedRegionalUsageSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalUsageSnapshot>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional usage snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageSnapshotQuery {
    pub scope: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalUsageSnapshotQuery {
    pub fn matches(&self, item: &RegionalUsageSnapshot) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalUsageSnapshot]) -> Vec<RegionalUsageSnapshot> {
        let matched = collections::filter(items, |candidate: &RegionalUsageSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_usage_snapshots(
    store: Rc<dyn RegionalUsageSnapshotStore>,
    query: &RegionalUsageSnapshotQuery,
) -> Vec<RegionalUsageSnapshot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalUsageSnapshotStore> = new_memory_regional_usage_snapshot_store();
        let item = new_regional_usage_snapshot("code-678", "name-181");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
