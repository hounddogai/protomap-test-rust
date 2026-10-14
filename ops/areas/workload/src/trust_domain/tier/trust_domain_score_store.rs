use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps trust domain scores by code.
pub trait TrustDomainScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TrustDomainScore>;
    fn save(&self, item: &TrustDomainScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TrustDomainScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTrustDomainScoreStore {
    pub items: RefCell<BTreeMap<String, TrustDomainScore>>,
    pub order: RefCell<Vec<String>>,
}

impl TrustDomainScoreStore for MemoryTrustDomainScoreStore {
    fn find(&self, code: &str) -> Option<TrustDomainScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TrustDomainScore) {
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

    fn list(&self) -> Vec<TrustDomainScore> {
        let mut out: Vec<TrustDomainScore> = Vec::new();
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

pub fn new_memory_trust_domain_score_store() -> Rc<dyn TrustDomainScoreStore> {
    Rc::new(MemoryTrustDomainScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTrustDomainScoreStore {
    pub inner: Rc<dyn TrustDomainScoreStore>,
    pub cache: RefCell<cache::TtlCache<TrustDomainScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TrustDomainScoreStore for CachedTrustDomainScoreStore {
    fn find(&self, code: &str) -> Option<TrustDomainScore> {
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

    fn save(&self, item: &TrustDomainScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TrustDomainScore> {
        self.inner.list()
    }
}

pub fn new_cached_trust_domain_score_store(
    inner: Rc<dyn TrustDomainScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TrustDomainScoreStore> {
    Rc::new(CachedTrustDomainScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TrustDomainScore>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects trust domain scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainScoreQuery {
    pub label: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TrustDomainScoreQuery {
    pub fn matches(&self, item: &TrustDomainScore) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TrustDomainScore]) -> Vec<TrustDomainScore> {
        let matched = collections::filter(items, |candidate: &TrustDomainScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_trust_domain_scores(
    store: Rc<dyn TrustDomainScoreStore>,
    query: &TrustDomainScoreQuery,
) -> Vec<TrustDomainScore> {
    query.apply(&store.list())
}
