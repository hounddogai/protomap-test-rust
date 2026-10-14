use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending flow log estimates by code.
pub trait PendingFlowLogEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingFlowLogEstimate>;
    fn save(&self, item: &PendingFlowLogEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingFlowLogEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingFlowLogEstimateStore {
    pub items: RefCell<BTreeMap<String, PendingFlowLogEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingFlowLogEstimateStore for MemoryPendingFlowLogEstimateStore {
    fn find(&self, code: &str) -> Option<PendingFlowLogEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingFlowLogEstimate) {
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

    fn list(&self) -> Vec<PendingFlowLogEstimate> {
        let mut out: Vec<PendingFlowLogEstimate> = Vec::new();
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

pub fn new_memory_pending_flow_log_estimate_store() -> Rc<dyn PendingFlowLogEstimateStore> {
    Rc::new(MemoryPendingFlowLogEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingFlowLogEstimateStore {
    pub inner: Rc<dyn PendingFlowLogEstimateStore>,
    pub cache: RefCell<cache::TtlCache<PendingFlowLogEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingFlowLogEstimateStore for CachedPendingFlowLogEstimateStore {
    fn find(&self, code: &str) -> Option<PendingFlowLogEstimate> {
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

    fn save(&self, item: &PendingFlowLogEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingFlowLogEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_pending_flow_log_estimate_store(
    inner: Rc<dyn PendingFlowLogEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingFlowLogEstimateStore> {
    Rc::new(CachedPendingFlowLogEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingFlowLogEstimate>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending flow log estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogEstimateQuery {
    pub title: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingFlowLogEstimateQuery {
    pub fn matches(&self, item: &PendingFlowLogEstimate) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[PendingFlowLogEstimate]) -> Vec<PendingFlowLogEstimate> {
        let matched = collections::filter(items, |candidate: &PendingFlowLogEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_flow_log_estimates(
    store: Rc<dyn PendingFlowLogEstimateStore>,
    query: &PendingFlowLogEstimateQuery,
) -> Vec<PendingFlowLogEstimate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PendingFlowLogEstimateStore> = new_memory_pending_flow_log_estimate_store();
        let item = new_pending_flow_log_estimate("code-572", "title-914");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
