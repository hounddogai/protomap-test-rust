use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps flow log preferences by code.
pub trait FlowLogPreferenceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FlowLogPreference>;
    fn save(&self, item: &FlowLogPreference);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FlowLogPreference>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFlowLogPreferenceStore {
    pub items: RefCell<BTreeMap<String, FlowLogPreference>>,
    pub order: RefCell<Vec<String>>,
}

impl FlowLogPreferenceStore for MemoryFlowLogPreferenceStore {
    fn find(&self, code: &str) -> Option<FlowLogPreference> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FlowLogPreference) {
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

    fn list(&self) -> Vec<FlowLogPreference> {
        let mut out: Vec<FlowLogPreference> = Vec::new();
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

pub fn new_memory_flow_log_preference_store() -> Rc<dyn FlowLogPreferenceStore> {
    Rc::new(MemoryFlowLogPreferenceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFlowLogPreferenceStore {
    pub inner: Rc<dyn FlowLogPreferenceStore>,
    pub cache: RefCell<cache::TtlCache<FlowLogPreference>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FlowLogPreferenceStore for CachedFlowLogPreferenceStore {
    fn find(&self, code: &str) -> Option<FlowLogPreference> {
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

    fn save(&self, item: &FlowLogPreference) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FlowLogPreference> {
        self.inner.list()
    }
}

pub fn new_cached_flow_log_preference_store(
    inner: Rc<dyn FlowLogPreferenceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FlowLogPreferenceStore> {
    Rc::new(CachedFlowLogPreferenceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FlowLogPreference>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects flow log preferences. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPreferenceQuery {
    pub country: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FlowLogPreferenceQuery {
    pub fn matches(&self, item: &FlowLogPreference) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FlowLogPreference]) -> Vec<FlowLogPreference> {
        let matched = collections::filter(items, |candidate: &FlowLogPreference| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_flow_log_preferences(
    store: Rc<dyn FlowLogPreferenceStore>,
    query: &FlowLogPreferenceQuery,
) -> Vec<FlowLogPreference> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FlowLogPreferenceStore> = new_memory_flow_log_preference_store();
        let item = new_flow_log_preference("code-532", "name-28");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
