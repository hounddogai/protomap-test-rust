use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled egress policies by code.
pub trait ScheduledEgressPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledEgressPolicy>;
    fn save(&self, item: &ScheduledEgressPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledEgressPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledEgressPolicyStore {
    pub items: RefCell<BTreeMap<String, ScheduledEgressPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledEgressPolicyStore for MemoryScheduledEgressPolicyStore {
    fn find(&self, code: &str) -> Option<ScheduledEgressPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledEgressPolicy) {
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

    fn list(&self) -> Vec<ScheduledEgressPolicy> {
        let mut out: Vec<ScheduledEgressPolicy> = Vec::new();
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

pub fn new_memory_scheduled_egress_policy_store() -> Rc<dyn ScheduledEgressPolicyStore> {
    Rc::new(MemoryScheduledEgressPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledEgressPolicyStore {
    pub inner: Rc<dyn ScheduledEgressPolicyStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledEgressPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledEgressPolicyStore for CachedScheduledEgressPolicyStore {
    fn find(&self, code: &str) -> Option<ScheduledEgressPolicy> {
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

    fn save(&self, item: &ScheduledEgressPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledEgressPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_egress_policy_store(
    inner: Rc<dyn ScheduledEgressPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledEgressPolicyStore> {
    Rc::new(CachedScheduledEgressPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledEgressPolicy>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled egress policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressPolicyQuery {
    pub cluster: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledEgressPolicyQuery {
    pub fn matches(&self, item: &ScheduledEgressPolicy) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledEgressPolicy]) -> Vec<ScheduledEgressPolicy> {
        let matched = collections::filter(items, |candidate: &ScheduledEgressPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_egress_policies(
    store: Rc<dyn ScheduledEgressPolicyStore>,
    query: &ScheduledEgressPolicyQuery,
) -> Vec<ScheduledEgressPolicy> {
    query.apply(&store.list())
}
