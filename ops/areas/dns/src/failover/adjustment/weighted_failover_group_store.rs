use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted failover groups by code.
pub trait WeightedFailoverGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedFailoverGroup>;
    fn save(&self, item: &WeightedFailoverGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedFailoverGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedFailoverGroupStore {
    pub items: RefCell<BTreeMap<String, WeightedFailoverGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedFailoverGroupStore for MemoryWeightedFailoverGroupStore {
    fn find(&self, code: &str) -> Option<WeightedFailoverGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedFailoverGroup) {
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

    fn list(&self) -> Vec<WeightedFailoverGroup> {
        let mut out: Vec<WeightedFailoverGroup> = Vec::new();
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

pub fn new_memory_weighted_failover_group_store() -> Rc<dyn WeightedFailoverGroupStore> {
    Rc::new(MemoryWeightedFailoverGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedFailoverGroupStore {
    pub inner: Rc<dyn WeightedFailoverGroupStore>,
    pub cache: RefCell<cache::TtlCache<WeightedFailoverGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedFailoverGroupStore for CachedWeightedFailoverGroupStore {
    fn find(&self, code: &str) -> Option<WeightedFailoverGroup> {
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

    fn save(&self, item: &WeightedFailoverGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedFailoverGroup> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_failover_group_store(
    inner: Rc<dyn WeightedFailoverGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedFailoverGroupStore> {
    Rc::new(CachedWeightedFailoverGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedFailoverGroup>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted failover groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverGroupQuery {
    pub source: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedFailoverGroupQuery {
    pub fn matches(&self, item: &WeightedFailoverGroup) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
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

    pub fn apply(&self, items: &[WeightedFailoverGroup]) -> Vec<WeightedFailoverGroup> {
        let matched = collections::filter(items, |candidate: &WeightedFailoverGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_failover_groups(
    store: Rc<dyn WeightedFailoverGroupStore>,
    query: &WeightedFailoverGroupQuery,
) -> Vec<WeightedFailoverGroup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedFailoverGroupStore> = new_memory_weighted_failover_group_store();
        let item = new_weighted_failover_group("code-1", "label-629");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
