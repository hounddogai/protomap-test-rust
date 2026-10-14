use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered flow log preferences by code.
pub trait TieredFlowLogPreferenceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredFlowLogPreference>;
    fn save(&self, item: &TieredFlowLogPreference);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredFlowLogPreference>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredFlowLogPreferenceStore {
    pub items: RefCell<BTreeMap<String, TieredFlowLogPreference>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredFlowLogPreferenceStore for MemoryTieredFlowLogPreferenceStore {
    fn find(&self, code: &str) -> Option<TieredFlowLogPreference> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredFlowLogPreference) {
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

    fn list(&self) -> Vec<TieredFlowLogPreference> {
        let mut out: Vec<TieredFlowLogPreference> = Vec::new();
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

pub fn new_memory_tiered_flow_log_preference_store() -> Rc<dyn TieredFlowLogPreferenceStore> {
    Rc::new(MemoryTieredFlowLogPreferenceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredFlowLogPreferenceStore {
    pub inner: Rc<dyn TieredFlowLogPreferenceStore>,
    pub cache: RefCell<cache::TtlCache<TieredFlowLogPreference>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredFlowLogPreferenceStore for CachedTieredFlowLogPreferenceStore {
    fn find(&self, code: &str) -> Option<TieredFlowLogPreference> {
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

    fn save(&self, item: &TieredFlowLogPreference) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredFlowLogPreference> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_flow_log_preference_store(
    inner: Rc<dyn TieredFlowLogPreferenceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredFlowLogPreferenceStore> {
    Rc::new(CachedTieredFlowLogPreferenceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredFlowLogPreference>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered flow log preferences. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogPreferenceQuery {
    pub namespace: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredFlowLogPreferenceQuery {
    pub fn matches(&self, item: &TieredFlowLogPreference) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredFlowLogPreference]) -> Vec<TieredFlowLogPreference> {
        let matched = collections::filter(items, |candidate: &TieredFlowLogPreference| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_flow_log_preferences(
    store: Rc<dyn TieredFlowLogPreferenceStore>,
    query: &TieredFlowLogPreferenceQuery,
) -> Vec<TieredFlowLogPreference> {
    query.apply(&store.list())
}
