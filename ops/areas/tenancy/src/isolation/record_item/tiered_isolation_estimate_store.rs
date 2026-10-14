use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered isolation estimates by code.
pub trait TieredIsolationEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredIsolationEstimate>;
    fn save(&self, item: &TieredIsolationEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredIsolationEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredIsolationEstimateStore {
    pub items: RefCell<BTreeMap<String, TieredIsolationEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredIsolationEstimateStore for MemoryTieredIsolationEstimateStore {
    fn find(&self, code: &str) -> Option<TieredIsolationEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredIsolationEstimate) {
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

    fn list(&self) -> Vec<TieredIsolationEstimate> {
        let mut out: Vec<TieredIsolationEstimate> = Vec::new();
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

pub fn new_memory_tiered_isolation_estimate_store() -> Rc<dyn TieredIsolationEstimateStore> {
    Rc::new(MemoryTieredIsolationEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredIsolationEstimateStore {
    pub inner: Rc<dyn TieredIsolationEstimateStore>,
    pub cache: RefCell<cache::TtlCache<TieredIsolationEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredIsolationEstimateStore for CachedTieredIsolationEstimateStore {
    fn find(&self, code: &str) -> Option<TieredIsolationEstimate> {
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

    fn save(&self, item: &TieredIsolationEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredIsolationEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_isolation_estimate_store(
    inner: Rc<dyn TieredIsolationEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredIsolationEstimateStore> {
    Rc::new(CachedTieredIsolationEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredIsolationEstimate>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered isolation estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationEstimateQuery {
    pub category: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredIsolationEstimateQuery {
    pub fn matches(&self, item: &TieredIsolationEstimate) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredIsolationEstimate]) -> Vec<TieredIsolationEstimate> {
        let matched = collections::filter(items, |candidate: &TieredIsolationEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_isolation_estimates(
    store: Rc<dyn TieredIsolationEstimateStore>,
    query: &TieredIsolationEstimateQuery,
) -> Vec<TieredIsolationEstimate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredIsolationEstimateStore> = new_memory_tiered_isolation_estimate_store();
        let item = new_tiered_isolation_estimate("code-497", "title-281");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
