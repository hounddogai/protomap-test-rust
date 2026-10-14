use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy flow log policies by code.
pub trait LegacyFlowLogPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyFlowLogPolicy>;
    fn save(&self, item: &LegacyFlowLogPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyFlowLogPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyFlowLogPolicyStore {
    pub items: RefCell<BTreeMap<String, LegacyFlowLogPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyFlowLogPolicyStore for MemoryLegacyFlowLogPolicyStore {
    fn find(&self, code: &str) -> Option<LegacyFlowLogPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyFlowLogPolicy) {
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

    fn list(&self) -> Vec<LegacyFlowLogPolicy> {
        let mut out: Vec<LegacyFlowLogPolicy> = Vec::new();
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

pub fn new_memory_legacy_flow_log_policy_store() -> Rc<dyn LegacyFlowLogPolicyStore> {
    Rc::new(MemoryLegacyFlowLogPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyFlowLogPolicyStore {
    pub inner: Rc<dyn LegacyFlowLogPolicyStore>,
    pub cache: RefCell<cache::TtlCache<LegacyFlowLogPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyFlowLogPolicyStore for CachedLegacyFlowLogPolicyStore {
    fn find(&self, code: &str) -> Option<LegacyFlowLogPolicy> {
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

    fn save(&self, item: &LegacyFlowLogPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyFlowLogPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_flow_log_policy_store(
    inner: Rc<dyn LegacyFlowLogPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyFlowLogPolicyStore> {
    Rc::new(CachedLegacyFlowLogPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyFlowLogPolicy>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy flow log policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogPolicyQuery {
    pub namespace: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyFlowLogPolicyQuery {
    pub fn matches(&self, item: &LegacyFlowLogPolicy) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyFlowLogPolicy]) -> Vec<LegacyFlowLogPolicy> {
        let matched = collections::filter(items, |candidate: &LegacyFlowLogPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_flow_log_policies(
    store: Rc<dyn LegacyFlowLogPolicyStore>,
    query: &LegacyFlowLogPolicyQuery,
) -> Vec<LegacyFlowLogPolicy> {
    query.apply(&store.list())
}
