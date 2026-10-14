use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rule tiers by code.
pub trait RuleTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RuleTier>;
    fn save(&self, item: &RuleTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RuleTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRuleTierStore {
    pub items: RefCell<BTreeMap<String, RuleTier>>,
    pub order: RefCell<Vec<String>>,
}

impl RuleTierStore for MemoryRuleTierStore {
    fn find(&self, code: &str) -> Option<RuleTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RuleTier) {
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

    fn list(&self) -> Vec<RuleTier> {
        let mut out: Vec<RuleTier> = Vec::new();
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

pub fn new_memory_rule_tier_store() -> Rc<dyn RuleTierStore> {
    Rc::new(MemoryRuleTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRuleTierStore {
    pub inner: Rc<dyn RuleTierStore>,
    pub cache: RefCell<cache::TtlCache<RuleTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RuleTierStore for CachedRuleTierStore {
    fn find(&self, code: &str) -> Option<RuleTier> {
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

    fn save(&self, item: &RuleTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RuleTier> {
        self.inner.list()
    }
}

pub fn new_cached_rule_tier_store(
    inner: Rc<dyn RuleTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RuleTierStore> {
    Rc::new(CachedRuleTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RuleTier>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rule tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleTierQuery {
    pub cluster: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RuleTierQuery {
    pub fn matches(&self, item: &RuleTier) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RuleTier]) -> Vec<RuleTier> {
        let matched = collections::filter(items, |candidate: &RuleTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rule_tiers(store: Rc<dyn RuleTierStore>, query: &RuleTierQuery) -> Vec<RuleTier> {
    query.apply(&store.list())
}
