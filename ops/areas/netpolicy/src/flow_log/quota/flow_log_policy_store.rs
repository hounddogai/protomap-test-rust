use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps flow log policies by code.
pub trait FlowLogPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FlowLogPolicy>;
    fn save(&self, item: &FlowLogPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FlowLogPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFlowLogPolicyStore {
    pub items: RefCell<BTreeMap<String, FlowLogPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl FlowLogPolicyStore for MemoryFlowLogPolicyStore {
    fn find(&self, code: &str) -> Option<FlowLogPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FlowLogPolicy) {
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

    fn list(&self) -> Vec<FlowLogPolicy> {
        let mut out: Vec<FlowLogPolicy> = Vec::new();
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

pub fn new_memory_flow_log_policy_store() -> Rc<dyn FlowLogPolicyStore> {
    Rc::new(MemoryFlowLogPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFlowLogPolicyStore {
    pub inner: Rc<dyn FlowLogPolicyStore>,
    pub cache: RefCell<cache::TtlCache<FlowLogPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FlowLogPolicyStore for CachedFlowLogPolicyStore {
    fn find(&self, code: &str) -> Option<FlowLogPolicy> {
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

    fn save(&self, item: &FlowLogPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FlowLogPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_flow_log_policy_store(
    inner: Rc<dyn FlowLogPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FlowLogPolicyStore> {
    Rc::new(CachedFlowLogPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FlowLogPolicy>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects flow log policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPolicyQuery {
    pub cluster: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FlowLogPolicyQuery {
    pub fn matches(&self, item: &FlowLogPolicy) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FlowLogPolicy]) -> Vec<FlowLogPolicy> {
        let matched = collections::filter(items, |candidate: &FlowLogPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_flow_log_policies(store: Rc<dyn FlowLogPolicyStore>, query: &FlowLogPolicyQuery) -> Vec<FlowLogPolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FlowLogPolicyStore> = new_memory_flow_log_policy_store();
        let item = new_flow_log_policy("code-832", "title-967");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
