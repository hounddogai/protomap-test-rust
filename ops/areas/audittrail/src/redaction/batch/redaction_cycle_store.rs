use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps redaction cycles by code.
pub trait RedactionCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RedactionCycle>;
    fn save(&self, item: &RedactionCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RedactionCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRedactionCycleStore {
    pub items: RefCell<BTreeMap<String, RedactionCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl RedactionCycleStore for MemoryRedactionCycleStore {
    fn find(&self, code: &str) -> Option<RedactionCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RedactionCycle) {
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

    fn list(&self) -> Vec<RedactionCycle> {
        let mut out: Vec<RedactionCycle> = Vec::new();
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

pub fn new_memory_redaction_cycle_store() -> Rc<dyn RedactionCycleStore> {
    Rc::new(MemoryRedactionCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRedactionCycleStore {
    pub inner: Rc<dyn RedactionCycleStore>,
    pub cache: RefCell<cache::TtlCache<RedactionCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RedactionCycleStore for CachedRedactionCycleStore {
    fn find(&self, code: &str) -> Option<RedactionCycle> {
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

    fn save(&self, item: &RedactionCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RedactionCycle> {
        self.inner.list()
    }
}

pub fn new_cached_redaction_cycle_store(
    inner: Rc<dyn RedactionCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RedactionCycleStore> {
    Rc::new(CachedRedactionCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RedactionCycle>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects redaction cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionCycleQuery {
    pub title: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RedactionCycleQuery {
    pub fn matches(&self, item: &RedactionCycle) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RedactionCycle]) -> Vec<RedactionCycle> {
        let matched = collections::filter(items, |candidate: &RedactionCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_redaction_cycles(store: Rc<dyn RedactionCycleStore>, query: &RedactionCycleQuery) -> Vec<RedactionCycle> {
    query.apply(&store.list())
}
