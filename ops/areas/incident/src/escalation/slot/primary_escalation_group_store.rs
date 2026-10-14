use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary escalation groups by code.
pub trait PrimaryEscalationGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryEscalationGroup>;
    fn save(&self, item: &PrimaryEscalationGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryEscalationGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryEscalationGroupStore {
    pub items: RefCell<BTreeMap<String, PrimaryEscalationGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryEscalationGroupStore for MemoryPrimaryEscalationGroupStore {
    fn find(&self, code: &str) -> Option<PrimaryEscalationGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryEscalationGroup) {
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

    fn list(&self) -> Vec<PrimaryEscalationGroup> {
        let mut out: Vec<PrimaryEscalationGroup> = Vec::new();
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

pub fn new_memory_primary_escalation_group_store() -> Rc<dyn PrimaryEscalationGroupStore> {
    Rc::new(MemoryPrimaryEscalationGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryEscalationGroupStore {
    pub inner: Rc<dyn PrimaryEscalationGroupStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryEscalationGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryEscalationGroupStore for CachedPrimaryEscalationGroupStore {
    fn find(&self, code: &str) -> Option<PrimaryEscalationGroup> {
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

    fn save(&self, item: &PrimaryEscalationGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryEscalationGroup> {
        self.inner.list()
    }
}

pub fn new_cached_primary_escalation_group_store(
    inner: Rc<dyn PrimaryEscalationGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryEscalationGroupStore> {
    Rc::new(CachedPrimaryEscalationGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryEscalationGroup>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary escalation groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationGroupQuery {
    pub source: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryEscalationGroupQuery {
    pub fn matches(&self, item: &PrimaryEscalationGroup) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryEscalationGroup]) -> Vec<PrimaryEscalationGroup> {
        let matched = collections::filter(items, |candidate: &PrimaryEscalationGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_escalation_groups(
    store: Rc<dyn PrimaryEscalationGroupStore>,
    query: &PrimaryEscalationGroupQuery,
) -> Vec<PrimaryEscalationGroup> {
    query.apply(&store.list())
}
