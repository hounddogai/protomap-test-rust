use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default escalation rollups by code.
pub trait DefaultEscalationRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultEscalationRollup>;
    fn save(&self, item: &DefaultEscalationRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultEscalationRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultEscalationRollupStore {
    pub items: RefCell<BTreeMap<String, DefaultEscalationRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultEscalationRollupStore for MemoryDefaultEscalationRollupStore {
    fn find(&self, code: &str) -> Option<DefaultEscalationRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultEscalationRollup) {
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

    fn list(&self) -> Vec<DefaultEscalationRollup> {
        let mut out: Vec<DefaultEscalationRollup> = Vec::new();
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

pub fn new_memory_default_escalation_rollup_store() -> Rc<dyn DefaultEscalationRollupStore> {
    Rc::new(MemoryDefaultEscalationRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultEscalationRollupStore {
    pub inner: Rc<dyn DefaultEscalationRollupStore>,
    pub cache: RefCell<cache::TtlCache<DefaultEscalationRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultEscalationRollupStore for CachedDefaultEscalationRollupStore {
    fn find(&self, code: &str) -> Option<DefaultEscalationRollup> {
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

    fn save(&self, item: &DefaultEscalationRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultEscalationRollup> {
        self.inner.list()
    }
}

pub fn new_cached_default_escalation_rollup_store(
    inner: Rc<dyn DefaultEscalationRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultEscalationRollupStore> {
    Rc::new(CachedDefaultEscalationRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultEscalationRollup>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default escalation rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEscalationRollupQuery {
    pub name: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultEscalationRollupQuery {
    pub fn matches(&self, item: &DefaultEscalationRollup) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultEscalationRollup]) -> Vec<DefaultEscalationRollup> {
        let matched = collections::filter(items, |candidate: &DefaultEscalationRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_escalation_rollups(
    store: Rc<dyn DefaultEscalationRollupStore>,
    query: &DefaultEscalationRollupQuery,
) -> Vec<DefaultEscalationRollup> {
    query.apply(&store.list())
}
