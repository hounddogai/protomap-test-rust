use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps action item policies by code.
pub trait ActionItemPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ActionItemPolicy>;
    fn save(&self, item: &ActionItemPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ActionItemPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryActionItemPolicyStore {
    pub items: RefCell<BTreeMap<String, ActionItemPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl ActionItemPolicyStore for MemoryActionItemPolicyStore {
    fn find(&self, code: &str) -> Option<ActionItemPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ActionItemPolicy) {
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

    fn list(&self) -> Vec<ActionItemPolicy> {
        let mut out: Vec<ActionItemPolicy> = Vec::new();
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

pub fn new_memory_action_item_policy_store() -> Rc<dyn ActionItemPolicyStore> {
    Rc::new(MemoryActionItemPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedActionItemPolicyStore {
    pub inner: Rc<dyn ActionItemPolicyStore>,
    pub cache: RefCell<cache::TtlCache<ActionItemPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ActionItemPolicyStore for CachedActionItemPolicyStore {
    fn find(&self, code: &str) -> Option<ActionItemPolicy> {
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

    fn save(&self, item: &ActionItemPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ActionItemPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_action_item_policy_store(
    inner: Rc<dyn ActionItemPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ActionItemPolicyStore> {
    Rc::new(CachedActionItemPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ActionItemPolicy>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects action item policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPolicyQuery {
    pub scope: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ActionItemPolicyQuery {
    pub fn matches(&self, item: &ActionItemPolicy) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
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

    pub fn apply(&self, items: &[ActionItemPolicy]) -> Vec<ActionItemPolicy> {
        let matched = collections::filter(items, |candidate: &ActionItemPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_action_item_policies(
    store: Rc<dyn ActionItemPolicyStore>,
    query: &ActionItemPolicyQuery,
) -> Vec<ActionItemPolicy> {
    query.apply(&store.list())
}
