use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps flow log variances by code.
pub trait FlowLogVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FlowLogVariance>;
    fn save(&self, item: &FlowLogVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FlowLogVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFlowLogVarianceStore {
    pub items: RefCell<BTreeMap<String, FlowLogVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl FlowLogVarianceStore for MemoryFlowLogVarianceStore {
    fn find(&self, code: &str) -> Option<FlowLogVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FlowLogVariance) {
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

    fn list(&self) -> Vec<FlowLogVariance> {
        let mut out: Vec<FlowLogVariance> = Vec::new();
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

pub fn new_memory_flow_log_variance_store() -> Rc<dyn FlowLogVarianceStore> {
    Rc::new(MemoryFlowLogVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFlowLogVarianceStore {
    pub inner: Rc<dyn FlowLogVarianceStore>,
    pub cache: RefCell<cache::TtlCache<FlowLogVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FlowLogVarianceStore for CachedFlowLogVarianceStore {
    fn find(&self, code: &str) -> Option<FlowLogVariance> {
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

    fn save(&self, item: &FlowLogVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FlowLogVariance> {
        self.inner.list()
    }
}

pub fn new_cached_flow_log_variance_store(
    inner: Rc<dyn FlowLogVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FlowLogVarianceStore> {
    Rc::new(CachedFlowLogVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FlowLogVariance>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects flow log variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogVarianceQuery {
    pub name: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FlowLogVarianceQuery {
    pub fn matches(&self, item: &FlowLogVariance) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[FlowLogVariance]) -> Vec<FlowLogVariance> {
        let matched = collections::filter(items, |candidate: &FlowLogVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_flow_log_variances(
    store: Rc<dyn FlowLogVarianceStore>,
    query: &FlowLogVarianceQuery,
) -> Vec<FlowLogVariance> {
    query.apply(&store.list())
}
