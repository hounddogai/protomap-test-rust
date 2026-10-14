use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted escalation tiers by code.
pub trait WeightedEscalationTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedEscalationTier>;
    fn save(&self, item: &WeightedEscalationTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedEscalationTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedEscalationTierStore {
    pub items: RefCell<BTreeMap<String, WeightedEscalationTier>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedEscalationTierStore for MemoryWeightedEscalationTierStore {
    fn find(&self, code: &str) -> Option<WeightedEscalationTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedEscalationTier) {
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

    fn list(&self) -> Vec<WeightedEscalationTier> {
        let mut out: Vec<WeightedEscalationTier> = Vec::new();
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

pub fn new_memory_weighted_escalation_tier_store() -> Rc<dyn WeightedEscalationTierStore> {
    Rc::new(MemoryWeightedEscalationTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedEscalationTierStore {
    pub inner: Rc<dyn WeightedEscalationTierStore>,
    pub cache: RefCell<cache::TtlCache<WeightedEscalationTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedEscalationTierStore for CachedWeightedEscalationTierStore {
    fn find(&self, code: &str) -> Option<WeightedEscalationTier> {
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

    fn save(&self, item: &WeightedEscalationTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedEscalationTier> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_escalation_tier_store(
    inner: Rc<dyn WeightedEscalationTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedEscalationTierStore> {
    Rc::new(CachedWeightedEscalationTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedEscalationTier>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted escalation tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEscalationTierQuery {
    pub region: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedEscalationTierQuery {
    pub fn matches(&self, item: &WeightedEscalationTier) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[WeightedEscalationTier]) -> Vec<WeightedEscalationTier> {
        let matched = collections::filter(items, |candidate: &WeightedEscalationTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_escalation_tiers(
    store: Rc<dyn WeightedEscalationTierStore>,
    query: &WeightedEscalationTierQuery,
) -> Vec<WeightedEscalationTier> {
    query.apply(&store.list())
}
