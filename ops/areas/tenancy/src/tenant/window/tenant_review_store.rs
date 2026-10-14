use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tenant reviews by code.
pub trait TenantReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TenantReview>;
    fn save(&self, item: &TenantReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TenantReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTenantReviewStore {
    pub items: RefCell<BTreeMap<String, TenantReview>>,
    pub order: RefCell<Vec<String>>,
}

impl TenantReviewStore for MemoryTenantReviewStore {
    fn find(&self, code: &str) -> Option<TenantReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TenantReview) {
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

    fn list(&self) -> Vec<TenantReview> {
        let mut out: Vec<TenantReview> = Vec::new();
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

pub fn new_memory_tenant_review_store() -> Rc<dyn TenantReviewStore> {
    Rc::new(MemoryTenantReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTenantReviewStore {
    pub inner: Rc<dyn TenantReviewStore>,
    pub cache: RefCell<cache::TtlCache<TenantReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TenantReviewStore for CachedTenantReviewStore {
    fn find(&self, code: &str) -> Option<TenantReview> {
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

    fn save(&self, item: &TenantReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TenantReview> {
        self.inner.list()
    }
}

pub fn new_cached_tenant_review_store(
    inner: Rc<dyn TenantReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TenantReviewStore> {
    Rc::new(CachedTenantReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TenantReview>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tenant reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantReviewQuery {
    pub locale: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TenantReviewQuery {
    pub fn matches(&self, item: &TenantReview) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TenantReview]) -> Vec<TenantReview> {
        let matched = collections::filter(items, |candidate: &TenantReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tenant_reviews(store: Rc<dyn TenantReviewStore>, query: &TenantReviewQuery) -> Vec<TenantReview> {
    query.apply(&store.list())
}
