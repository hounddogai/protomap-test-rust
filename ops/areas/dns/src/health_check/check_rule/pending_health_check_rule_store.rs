use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending health check rules by code.
pub trait PendingHealthCheckRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingHealthCheckRule>;
    fn save(&self, item: &PendingHealthCheckRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingHealthCheckRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingHealthCheckRuleStore {
    pub items: RefCell<BTreeMap<String, PendingHealthCheckRule>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingHealthCheckRuleStore for MemoryPendingHealthCheckRuleStore {
    fn find(&self, code: &str) -> Option<PendingHealthCheckRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingHealthCheckRule) {
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

    fn list(&self) -> Vec<PendingHealthCheckRule> {
        let mut out: Vec<PendingHealthCheckRule> = Vec::new();
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

pub fn new_memory_pending_health_check_rule_store() -> Rc<dyn PendingHealthCheckRuleStore> {
    Rc::new(MemoryPendingHealthCheckRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingHealthCheckRuleStore {
    pub inner: Rc<dyn PendingHealthCheckRuleStore>,
    pub cache: RefCell<cache::TtlCache<PendingHealthCheckRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingHealthCheckRuleStore for CachedPendingHealthCheckRuleStore {
    fn find(&self, code: &str) -> Option<PendingHealthCheckRule> {
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

    fn save(&self, item: &PendingHealthCheckRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingHealthCheckRule> {
        self.inner.list()
    }
}

pub fn new_cached_pending_health_check_rule_store(
    inner: Rc<dyn PendingHealthCheckRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingHealthCheckRuleStore> {
    Rc::new(CachedPendingHealthCheckRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingHealthCheckRule>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending health check rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckRuleQuery {
    pub segment: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingHealthCheckRuleQuery {
    pub fn matches(&self, item: &PendingHealthCheckRule) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PendingHealthCheckRule]) -> Vec<PendingHealthCheckRule> {
        let matched = collections::filter(items, |candidate: &PendingHealthCheckRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_health_check_rules(
    store: Rc<dyn PendingHealthCheckRuleStore>,
    query: &PendingHealthCheckRuleQuery,
) -> Vec<PendingHealthCheckRule> {
    query.apply(&store.list())
}
