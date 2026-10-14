use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps flow log agreements by code.
pub trait FlowLogAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FlowLogAgreement>;
    fn save(&self, item: &FlowLogAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FlowLogAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFlowLogAgreementStore {
    pub items: RefCell<BTreeMap<String, FlowLogAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl FlowLogAgreementStore for MemoryFlowLogAgreementStore {
    fn find(&self, code: &str) -> Option<FlowLogAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FlowLogAgreement) {
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

    fn list(&self) -> Vec<FlowLogAgreement> {
        let mut out: Vec<FlowLogAgreement> = Vec::new();
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

pub fn new_memory_flow_log_agreement_store() -> Rc<dyn FlowLogAgreementStore> {
    Rc::new(MemoryFlowLogAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFlowLogAgreementStore {
    pub inner: Rc<dyn FlowLogAgreementStore>,
    pub cache: RefCell<cache::TtlCache<FlowLogAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FlowLogAgreementStore for CachedFlowLogAgreementStore {
    fn find(&self, code: &str) -> Option<FlowLogAgreement> {
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

    fn save(&self, item: &FlowLogAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FlowLogAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_flow_log_agreement_store(
    inner: Rc<dyn FlowLogAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FlowLogAgreementStore> {
    Rc::new(CachedFlowLogAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FlowLogAgreement>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects flow log agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAgreementQuery {
    pub namespace: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FlowLogAgreementQuery {
    pub fn matches(&self, item: &FlowLogAgreement) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.ceiling < self.min_ceiling {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FlowLogAgreement]) -> Vec<FlowLogAgreement> {
        let matched = collections::filter(items, |candidate: &FlowLogAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_flow_log_agreements(
    store: Rc<dyn FlowLogAgreementStore>,
    query: &FlowLogAgreementQuery,
) -> Vec<FlowLogAgreement> {
    query.apply(&store.list())
}
