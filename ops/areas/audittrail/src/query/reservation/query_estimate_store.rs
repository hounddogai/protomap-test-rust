use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps query estimates by code.
pub trait QueryEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QueryEstimate>;
    fn save(&self, item: &QueryEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QueryEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQueryEstimateStore {
    pub items: RefCell<BTreeMap<String, QueryEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl QueryEstimateStore for MemoryQueryEstimateStore {
    fn find(&self, code: &str) -> Option<QueryEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QueryEstimate) {
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

    fn list(&self) -> Vec<QueryEstimate> {
        let mut out: Vec<QueryEstimate> = Vec::new();
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

pub fn new_memory_query_estimate_store() -> Rc<dyn QueryEstimateStore> {
    Rc::new(MemoryQueryEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQueryEstimateStore {
    pub inner: Rc<dyn QueryEstimateStore>,
    pub cache: RefCell<cache::TtlCache<QueryEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QueryEstimateStore for CachedQueryEstimateStore {
    fn find(&self, code: &str) -> Option<QueryEstimate> {
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

    fn save(&self, item: &QueryEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QueryEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_query_estimate_store(
    inner: Rc<dyn QueryEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QueryEstimateStore> {
    Rc::new(CachedQueryEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QueryEstimate>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects query estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryEstimateQuery {
    pub title: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QueryEstimateQuery {
    pub fn matches(&self, item: &QueryEstimate) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[QueryEstimate]) -> Vec<QueryEstimate> {
        let matched = collections::filter(items, |candidate: &QueryEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_query_estimates(store: Rc<dyn QueryEstimateStore>, query: &QueryEstimateQuery) -> Vec<QueryEstimate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn QueryEstimateStore> = new_memory_query_estimate_store();
        let item = new_query_estimate("code-745", "title-139");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
