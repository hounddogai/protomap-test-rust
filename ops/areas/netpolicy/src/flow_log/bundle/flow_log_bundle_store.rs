use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps flow log bundles by code.
pub trait FlowLogBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FlowLogBundle>;
    fn save(&self, item: &FlowLogBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FlowLogBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFlowLogBundleStore {
    pub items: RefCell<BTreeMap<String, FlowLogBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl FlowLogBundleStore for MemoryFlowLogBundleStore {
    fn find(&self, code: &str) -> Option<FlowLogBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FlowLogBundle) {
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

    fn list(&self) -> Vec<FlowLogBundle> {
        let mut out: Vec<FlowLogBundle> = Vec::new();
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

pub fn new_memory_flow_log_bundle_store() -> Rc<dyn FlowLogBundleStore> {
    Rc::new(MemoryFlowLogBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFlowLogBundleStore {
    pub inner: Rc<dyn FlowLogBundleStore>,
    pub cache: RefCell<cache::TtlCache<FlowLogBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FlowLogBundleStore for CachedFlowLogBundleStore {
    fn find(&self, code: &str) -> Option<FlowLogBundle> {
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

    fn save(&self, item: &FlowLogBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FlowLogBundle> {
        self.inner.list()
    }
}

pub fn new_cached_flow_log_bundle_store(
    inner: Rc<dyn FlowLogBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FlowLogBundleStore> {
    Rc::new(CachedFlowLogBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FlowLogBundle>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects flow log bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBundleQuery {
    pub region: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FlowLogBundleQuery {
    pub fn matches(&self, item: &FlowLogBundle) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FlowLogBundle]) -> Vec<FlowLogBundle> {
        let matched = collections::filter(items, |candidate: &FlowLogBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_flow_log_bundles(store: Rc<dyn FlowLogBundleStore>, query: &FlowLogBundleQuery) -> Vec<FlowLogBundle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FlowLogBundleStore> = new_memory_flow_log_bundle_store();
        let item = new_flow_log_bundle("code-70", "label-77");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
