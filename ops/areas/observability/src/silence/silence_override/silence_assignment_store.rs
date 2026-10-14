use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps silence assignments by code.
pub trait SilenceAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SilenceAssignment>;
    fn save(&self, item: &SilenceAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SilenceAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySilenceAssignmentStore {
    pub items: RefCell<BTreeMap<String, SilenceAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl SilenceAssignmentStore for MemorySilenceAssignmentStore {
    fn find(&self, code: &str) -> Option<SilenceAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SilenceAssignment) {
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

    fn list(&self) -> Vec<SilenceAssignment> {
        let mut out: Vec<SilenceAssignment> = Vec::new();
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

pub fn new_memory_silence_assignment_store() -> Rc<dyn SilenceAssignmentStore> {
    Rc::new(MemorySilenceAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSilenceAssignmentStore {
    pub inner: Rc<dyn SilenceAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<SilenceAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SilenceAssignmentStore for CachedSilenceAssignmentStore {
    fn find(&self, code: &str) -> Option<SilenceAssignment> {
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

    fn save(&self, item: &SilenceAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SilenceAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_silence_assignment_store(
    inner: Rc<dyn SilenceAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SilenceAssignmentStore> {
    Rc::new(CachedSilenceAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SilenceAssignment>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects silence assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAssignmentQuery {
    pub scope: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SilenceAssignmentQuery {
    pub fn matches(&self, item: &SilenceAssignment) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SilenceAssignment]) -> Vec<SilenceAssignment> {
        let matched = collections::filter(items, |candidate: &SilenceAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_silence_assignments(
    store: Rc<dyn SilenceAssignmentStore>,
    query: &SilenceAssignmentQuery,
) -> Vec<SilenceAssignment> {
    query.apply(&store.list())
}
