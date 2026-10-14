use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted rule quotas by code.
pub trait WeightedRuleQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedRuleQuota>;
    fn save(&self, item: &WeightedRuleQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedRuleQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedRuleQuotaStore {
    pub items: RefCell<BTreeMap<String, WeightedRuleQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedRuleQuotaStore for MemoryWeightedRuleQuotaStore {
    fn find(&self, code: &str) -> Option<WeightedRuleQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedRuleQuota) {
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

    fn list(&self) -> Vec<WeightedRuleQuota> {
        let mut out: Vec<WeightedRuleQuota> = Vec::new();
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

pub fn new_memory_weighted_rule_quota_store() -> Rc<dyn WeightedRuleQuotaStore> {
    Rc::new(MemoryWeightedRuleQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedRuleQuotaStore {
    pub inner: Rc<dyn WeightedRuleQuotaStore>,
    pub cache: RefCell<cache::TtlCache<WeightedRuleQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedRuleQuotaStore for CachedWeightedRuleQuotaStore {
    fn find(&self, code: &str) -> Option<WeightedRuleQuota> {
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

    fn save(&self, item: &WeightedRuleQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedRuleQuota> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_rule_quota_store(
    inner: Rc<dyn WeightedRuleQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedRuleQuotaStore> {
    Rc::new(CachedWeightedRuleQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedRuleQuota>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted rule quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleQuotaQuery {
    pub locale: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedRuleQuotaQuery {
    pub fn matches(&self, item: &WeightedRuleQuota) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
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

    pub fn apply(&self, items: &[WeightedRuleQuota]) -> Vec<WeightedRuleQuota> {
        let matched = collections::filter(items, |candidate: &WeightedRuleQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_rule_quotas(
    store: Rc<dyn WeightedRuleQuotaStore>,
    query: &WeightedRuleQuotaQuery,
) -> Vec<WeightedRuleQuota> {
    query.apply(&store.list())
}
