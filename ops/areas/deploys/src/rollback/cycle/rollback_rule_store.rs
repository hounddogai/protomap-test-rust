use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rollback rules by code.
pub trait RollbackRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollbackRule>;
    fn save(&self, item: &RollbackRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollbackRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollbackRuleStore {
    pub items: RefCell<BTreeMap<String, RollbackRule>>,
    pub order: RefCell<Vec<String>>,
}

impl RollbackRuleStore for MemoryRollbackRuleStore {
    fn find(&self, code: &str) -> Option<RollbackRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollbackRule) {
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

    fn list(&self) -> Vec<RollbackRule> {
        let mut out: Vec<RollbackRule> = Vec::new();
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

pub fn new_memory_rollback_rule_store() -> Rc<dyn RollbackRuleStore> {
    Rc::new(MemoryRollbackRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollbackRuleStore {
    pub inner: Rc<dyn RollbackRuleStore>,
    pub cache: RefCell<cache::TtlCache<RollbackRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollbackRuleStore for CachedRollbackRuleStore {
    fn find(&self, code: &str) -> Option<RollbackRule> {
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

    fn save(&self, item: &RollbackRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollbackRule> {
        self.inner.list()
    }
}

pub fn new_cached_rollback_rule_store(
    inner: Rc<dyn RollbackRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollbackRuleStore> {
    Rc::new(CachedRollbackRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollbackRule>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rollback rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRuleQuery {
    pub label: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollbackRuleQuery {
    pub fn matches(&self, item: &RollbackRule) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[RollbackRule]) -> Vec<RollbackRule> {
        let matched = collections::filter(items, |candidate: &RollbackRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rollback_rules(store: Rc<dyn RollbackRuleStore>, query: &RollbackRuleQuery) -> Vec<RollbackRule> {
    query.apply(&store.list())
}
