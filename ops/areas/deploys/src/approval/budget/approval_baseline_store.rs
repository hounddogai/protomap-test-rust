use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps approval baselines by code.
pub trait ApprovalBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ApprovalBaseline>;
    fn save(&self, item: &ApprovalBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ApprovalBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryApprovalBaselineStore {
    pub items: RefCell<BTreeMap<String, ApprovalBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl ApprovalBaselineStore for MemoryApprovalBaselineStore {
    fn find(&self, code: &str) -> Option<ApprovalBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ApprovalBaseline) {
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

    fn list(&self) -> Vec<ApprovalBaseline> {
        let mut out: Vec<ApprovalBaseline> = Vec::new();
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

pub fn new_memory_approval_baseline_store() -> Rc<dyn ApprovalBaselineStore> {
    Rc::new(MemoryApprovalBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedApprovalBaselineStore {
    pub inner: Rc<dyn ApprovalBaselineStore>,
    pub cache: RefCell<cache::TtlCache<ApprovalBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ApprovalBaselineStore for CachedApprovalBaselineStore {
    fn find(&self, code: &str) -> Option<ApprovalBaseline> {
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

    fn save(&self, item: &ApprovalBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ApprovalBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_approval_baseline_store(
    inner: Rc<dyn ApprovalBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ApprovalBaselineStore> {
    Rc::new(CachedApprovalBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ApprovalBaseline>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects approval baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBaselineQuery {
    pub country: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ApprovalBaselineQuery {
    pub fn matches(&self, item: &ApprovalBaseline) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
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

    pub fn apply(&self, items: &[ApprovalBaseline]) -> Vec<ApprovalBaseline> {
        let matched = collections::filter(items, |candidate: &ApprovalBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_approval_baselines(
    store: Rc<dyn ApprovalBaselineStore>,
    query: &ApprovalBaselineQuery,
) -> Vec<ApprovalBaseline> {
    query.apply(&store.list())
}
