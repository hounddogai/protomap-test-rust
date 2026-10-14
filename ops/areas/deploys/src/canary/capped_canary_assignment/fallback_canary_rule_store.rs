use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback canary rules by code.
pub trait FallbackCanaryRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackCanaryRule>;
    fn save(&self, item: &FallbackCanaryRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackCanaryRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackCanaryRuleStore {
    pub items: RefCell<BTreeMap<String, FallbackCanaryRule>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackCanaryRuleStore for MemoryFallbackCanaryRuleStore {
    fn find(&self, code: &str) -> Option<FallbackCanaryRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackCanaryRule) {
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

    fn list(&self) -> Vec<FallbackCanaryRule> {
        let mut out: Vec<FallbackCanaryRule> = Vec::new();
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

pub fn new_memory_fallback_canary_rule_store() -> Rc<dyn FallbackCanaryRuleStore> {
    Rc::new(MemoryFallbackCanaryRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackCanaryRuleStore {
    pub inner: Rc<dyn FallbackCanaryRuleStore>,
    pub cache: RefCell<cache::TtlCache<FallbackCanaryRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackCanaryRuleStore for CachedFallbackCanaryRuleStore {
    fn find(&self, code: &str) -> Option<FallbackCanaryRule> {
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

    fn save(&self, item: &FallbackCanaryRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackCanaryRule> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_canary_rule_store(
    inner: Rc<dyn FallbackCanaryRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackCanaryRuleStore> {
    Rc::new(CachedFallbackCanaryRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackCanaryRule>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback canary rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackCanaryRuleQuery {
    pub segment: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackCanaryRuleQuery {
    pub fn matches(&self, item: &FallbackCanaryRule) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackCanaryRule]) -> Vec<FallbackCanaryRule> {
        let matched = collections::filter(items, |candidate: &FallbackCanaryRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_canary_rules(
    store: Rc<dyn FallbackCanaryRuleStore>,
    query: &FallbackCanaryRuleQuery,
) -> Vec<FallbackCanaryRule> {
    query.apply(&store.list())
}
