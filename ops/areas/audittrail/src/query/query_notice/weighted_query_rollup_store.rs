use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted query rollups by code.
pub trait WeightedQueryRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedQueryRollup>;
    fn save(&self, item: &WeightedQueryRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedQueryRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedQueryRollupStore {
    pub items: RefCell<BTreeMap<String, WeightedQueryRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedQueryRollupStore for MemoryWeightedQueryRollupStore {
    fn find(&self, code: &str) -> Option<WeightedQueryRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedQueryRollup) {
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

    fn list(&self) -> Vec<WeightedQueryRollup> {
        let mut out: Vec<WeightedQueryRollup> = Vec::new();
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

pub fn new_memory_weighted_query_rollup_store() -> Rc<dyn WeightedQueryRollupStore> {
    Rc::new(MemoryWeightedQueryRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedQueryRollupStore {
    pub inner: Rc<dyn WeightedQueryRollupStore>,
    pub cache: RefCell<cache::TtlCache<WeightedQueryRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedQueryRollupStore for CachedWeightedQueryRollupStore {
    fn find(&self, code: &str) -> Option<WeightedQueryRollup> {
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

    fn save(&self, item: &WeightedQueryRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedQueryRollup> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_query_rollup_store(
    inner: Rc<dyn WeightedQueryRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedQueryRollupStore> {
    Rc::new(CachedWeightedQueryRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedQueryRollup>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted query rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQueryRollupQuery {
    pub title: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedQueryRollupQuery {
    pub fn matches(&self, item: &WeightedQueryRollup) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[WeightedQueryRollup]) -> Vec<WeightedQueryRollup> {
        let matched = collections::filter(items, |candidate: &WeightedQueryRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_query_rollups(
    store: Rc<dyn WeightedQueryRollupStore>,
    query: &WeightedQueryRollupQuery,
) -> Vec<WeightedQueryRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedQueryRollupStore> = new_memory_weighted_query_rollup_store();
        let item = new_weighted_query_rollup("code-181", "title-987");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
