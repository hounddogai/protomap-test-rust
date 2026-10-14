use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps credential adjustments by code.
pub trait CredentialAdjustmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CredentialAdjustment>;
    fn save(&self, item: &CredentialAdjustment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CredentialAdjustment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCredentialAdjustmentStore {
    pub items: RefCell<BTreeMap<String, CredentialAdjustment>>,
    pub order: RefCell<Vec<String>>,
}

impl CredentialAdjustmentStore for MemoryCredentialAdjustmentStore {
    fn find(&self, code: &str) -> Option<CredentialAdjustment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CredentialAdjustment) {
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

    fn list(&self) -> Vec<CredentialAdjustment> {
        let mut out: Vec<CredentialAdjustment> = Vec::new();
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

pub fn new_memory_credential_adjustment_store() -> Rc<dyn CredentialAdjustmentStore> {
    Rc::new(MemoryCredentialAdjustmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCredentialAdjustmentStore {
    pub inner: Rc<dyn CredentialAdjustmentStore>,
    pub cache: RefCell<cache::TtlCache<CredentialAdjustment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CredentialAdjustmentStore for CachedCredentialAdjustmentStore {
    fn find(&self, code: &str) -> Option<CredentialAdjustment> {
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

    fn save(&self, item: &CredentialAdjustment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CredentialAdjustment> {
        self.inner.list()
    }
}

pub fn new_cached_credential_adjustment_store(
    inner: Rc<dyn CredentialAdjustmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CredentialAdjustmentStore> {
    Rc::new(CachedCredentialAdjustmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CredentialAdjustment>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects credential adjustments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAdjustmentQuery {
    pub channel: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CredentialAdjustmentQuery {
    pub fn matches(&self, item: &CredentialAdjustment) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.ends_at < self.min_ends_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CredentialAdjustment]) -> Vec<CredentialAdjustment> {
        let matched = collections::filter(items, |candidate: &CredentialAdjustment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_credential_adjustments(
    store: Rc<dyn CredentialAdjustmentStore>,
    query: &CredentialAdjustmentQuery,
) -> Vec<CredentialAdjustment> {
    query.apply(&store.list())
}
