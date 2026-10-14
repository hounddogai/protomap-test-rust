use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual trust domain scores by code.
pub trait ManualTrustDomainScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualTrustDomainScore>;
    fn save(&self, item: &ManualTrustDomainScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualTrustDomainScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualTrustDomainScoreStore {
    pub items: RefCell<BTreeMap<String, ManualTrustDomainScore>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualTrustDomainScoreStore for MemoryManualTrustDomainScoreStore {
    fn find(&self, code: &str) -> Option<ManualTrustDomainScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualTrustDomainScore) {
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

    fn list(&self) -> Vec<ManualTrustDomainScore> {
        let mut out: Vec<ManualTrustDomainScore> = Vec::new();
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

pub fn new_memory_manual_trust_domain_score_store() -> Rc<dyn ManualTrustDomainScoreStore> {
    Rc::new(MemoryManualTrustDomainScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualTrustDomainScoreStore {
    pub inner: Rc<dyn ManualTrustDomainScoreStore>,
    pub cache: RefCell<cache::TtlCache<ManualTrustDomainScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualTrustDomainScoreStore for CachedManualTrustDomainScoreStore {
    fn find(&self, code: &str) -> Option<ManualTrustDomainScore> {
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

    fn save(&self, item: &ManualTrustDomainScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualTrustDomainScore> {
        self.inner.list()
    }
}

pub fn new_cached_manual_trust_domain_score_store(
    inner: Rc<dyn ManualTrustDomainScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualTrustDomainScoreStore> {
    Rc::new(CachedManualTrustDomainScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualTrustDomainScore>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual trust domain scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTrustDomainScoreQuery {
    pub unit: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualTrustDomainScoreQuery {
    pub fn matches(&self, item: &ManualTrustDomainScore) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.cost < self.min_cost {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualTrustDomainScore]) -> Vec<ManualTrustDomainScore> {
        let matched = collections::filter(items, |candidate: &ManualTrustDomainScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_trust_domain_scores(
    store: Rc<dyn ManualTrustDomainScoreStore>,
    query: &ManualTrustDomainScoreQuery,
) -> Vec<ManualTrustDomainScore> {
    query.apply(&store.list())
}
