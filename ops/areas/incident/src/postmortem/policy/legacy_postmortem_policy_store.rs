use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy postmortem policies by code.
pub trait LegacyPostmortemPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyPostmortemPolicy>;
    fn save(&self, item: &LegacyPostmortemPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyPostmortemPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyPostmortemPolicyStore {
    pub items: RefCell<BTreeMap<String, LegacyPostmortemPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyPostmortemPolicyStore for MemoryLegacyPostmortemPolicyStore {
    fn find(&self, code: &str) -> Option<LegacyPostmortemPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyPostmortemPolicy) {
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

    fn list(&self) -> Vec<LegacyPostmortemPolicy> {
        let mut out: Vec<LegacyPostmortemPolicy> = Vec::new();
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

pub fn new_memory_legacy_postmortem_policy_store() -> Rc<dyn LegacyPostmortemPolicyStore> {
    Rc::new(MemoryLegacyPostmortemPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyPostmortemPolicyStore {
    pub inner: Rc<dyn LegacyPostmortemPolicyStore>,
    pub cache: RefCell<cache::TtlCache<LegacyPostmortemPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyPostmortemPolicyStore for CachedLegacyPostmortemPolicyStore {
    fn find(&self, code: &str) -> Option<LegacyPostmortemPolicy> {
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

    fn save(&self, item: &LegacyPostmortemPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyPostmortemPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_postmortem_policy_store(
    inner: Rc<dyn LegacyPostmortemPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyPostmortemPolicyStore> {
    Rc::new(CachedLegacyPostmortemPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyPostmortemPolicy>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy postmortem policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPostmortemPolicyQuery {
    pub source: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyPostmortemPolicyQuery {
    pub fn matches(&self, item: &LegacyPostmortemPolicy) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyPostmortemPolicy]) -> Vec<LegacyPostmortemPolicy> {
        let matched = collections::filter(items, |candidate: &LegacyPostmortemPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_postmortem_policies(
    store: Rc<dyn LegacyPostmortemPolicyStore>,
    query: &LegacyPostmortemPolicyQuery,
) -> Vec<LegacyPostmortemPolicy> {
    query.apply(&store.list())
}
