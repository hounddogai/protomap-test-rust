use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy flow log thresholds by code.
pub trait LegacyFlowLogThresholdStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyFlowLogThreshold>;
    fn save(&self, item: &LegacyFlowLogThreshold);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyFlowLogThreshold>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyFlowLogThresholdStore {
    pub items: RefCell<BTreeMap<String, LegacyFlowLogThreshold>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyFlowLogThresholdStore for MemoryLegacyFlowLogThresholdStore {
    fn find(&self, code: &str) -> Option<LegacyFlowLogThreshold> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyFlowLogThreshold) {
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

    fn list(&self) -> Vec<LegacyFlowLogThreshold> {
        let mut out: Vec<LegacyFlowLogThreshold> = Vec::new();
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

pub fn new_memory_legacy_flow_log_threshold_store() -> Rc<dyn LegacyFlowLogThresholdStore> {
    Rc::new(MemoryLegacyFlowLogThresholdStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyFlowLogThresholdStore {
    pub inner: Rc<dyn LegacyFlowLogThresholdStore>,
    pub cache: RefCell<cache::TtlCache<LegacyFlowLogThreshold>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyFlowLogThresholdStore for CachedLegacyFlowLogThresholdStore {
    fn find(&self, code: &str) -> Option<LegacyFlowLogThreshold> {
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

    fn save(&self, item: &LegacyFlowLogThreshold) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyFlowLogThreshold> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_flow_log_threshold_store(
    inner: Rc<dyn LegacyFlowLogThresholdStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyFlowLogThresholdStore> {
    Rc::new(CachedLegacyFlowLogThresholdStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyFlowLogThreshold>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy flow log thresholds. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogThresholdQuery {
    pub label: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyFlowLogThresholdQuery {
    pub fn matches(&self, item: &LegacyFlowLogThreshold) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyFlowLogThreshold]) -> Vec<LegacyFlowLogThreshold> {
        let matched = collections::filter(items, |candidate: &LegacyFlowLogThreshold| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_flow_log_thresholds(
    store: Rc<dyn LegacyFlowLogThresholdStore>,
    query: &LegacyFlowLogThresholdQuery,
) -> Vec<LegacyFlowLogThreshold> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyFlowLogThresholdStore> = new_memory_legacy_flow_log_threshold_store();
        let item = new_legacy_flow_log_threshold("code-179", "label-165");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
