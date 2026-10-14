use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted rule snapshots by code.
pub trait WeightedRuleSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedRuleSnapshot>;
    fn save(&self, item: &WeightedRuleSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedRuleSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedRuleSnapshotStore {
    pub items: RefCell<BTreeMap<String, WeightedRuleSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedRuleSnapshotStore for MemoryWeightedRuleSnapshotStore {
    fn find(&self, code: &str) -> Option<WeightedRuleSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedRuleSnapshot) {
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

    fn list(&self) -> Vec<WeightedRuleSnapshot> {
        let mut out: Vec<WeightedRuleSnapshot> = Vec::new();
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

pub fn new_memory_weighted_rule_snapshot_store() -> Rc<dyn WeightedRuleSnapshotStore> {
    Rc::new(MemoryWeightedRuleSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedRuleSnapshotStore {
    pub inner: Rc<dyn WeightedRuleSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<WeightedRuleSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedRuleSnapshotStore for CachedWeightedRuleSnapshotStore {
    fn find(&self, code: &str) -> Option<WeightedRuleSnapshot> {
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

    fn save(&self, item: &WeightedRuleSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedRuleSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_rule_snapshot_store(
    inner: Rc<dyn WeightedRuleSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedRuleSnapshotStore> {
    Rc::new(CachedWeightedRuleSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedRuleSnapshot>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted rule snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleSnapshotQuery {
    pub title: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedRuleSnapshotQuery {
    pub fn matches(&self, item: &WeightedRuleSnapshot) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedRuleSnapshot]) -> Vec<WeightedRuleSnapshot> {
        let matched = collections::filter(items, |candidate: &WeightedRuleSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_rule_snapshots(
    store: Rc<dyn WeightedRuleSnapshotStore>,
    query: &WeightedRuleSnapshotQuery,
) -> Vec<WeightedRuleSnapshot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedRuleSnapshotStore> = new_memory_weighted_rule_snapshot_store();
        let item = new_weighted_rule_snapshot("code-621", "title-921");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
