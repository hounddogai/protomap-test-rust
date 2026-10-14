use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal flow log assignments by code.
pub trait SeasonalFlowLogAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalFlowLogAssignment>;
    fn save(&self, item: &SeasonalFlowLogAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalFlowLogAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalFlowLogAssignmentStore {
    pub items: RefCell<BTreeMap<String, SeasonalFlowLogAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalFlowLogAssignmentStore for MemorySeasonalFlowLogAssignmentStore {
    fn find(&self, code: &str) -> Option<SeasonalFlowLogAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalFlowLogAssignment) {
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

    fn list(&self) -> Vec<SeasonalFlowLogAssignment> {
        let mut out: Vec<SeasonalFlowLogAssignment> = Vec::new();
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

pub fn new_memory_seasonal_flow_log_assignment_store() -> Rc<dyn SeasonalFlowLogAssignmentStore> {
    Rc::new(MemorySeasonalFlowLogAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalFlowLogAssignmentStore {
    pub inner: Rc<dyn SeasonalFlowLogAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalFlowLogAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalFlowLogAssignmentStore for CachedSeasonalFlowLogAssignmentStore {
    fn find(&self, code: &str) -> Option<SeasonalFlowLogAssignment> {
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

    fn save(&self, item: &SeasonalFlowLogAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalFlowLogAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_flow_log_assignment_store(
    inner: Rc<dyn SeasonalFlowLogAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalFlowLogAssignmentStore> {
    Rc::new(CachedSeasonalFlowLogAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalFlowLogAssignment>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal flow log assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFlowLogAssignmentQuery {
    pub region: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalFlowLogAssignmentQuery {
    pub fn matches(&self, item: &SeasonalFlowLogAssignment) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[SeasonalFlowLogAssignment]) -> Vec<SeasonalFlowLogAssignment> {
        let matched = collections::filter(items, |candidate: &SeasonalFlowLogAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_flow_log_assignments(
    store: Rc<dyn SeasonalFlowLogAssignmentStore>,
    query: &SeasonalFlowLogAssignmentQuery,
) -> Vec<SeasonalFlowLogAssignment> {
    query.apply(&store.list())
}
