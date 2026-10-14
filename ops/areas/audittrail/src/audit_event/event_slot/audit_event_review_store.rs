use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps audit event reviews by code.
pub trait AuditEventReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AuditEventReview>;
    fn save(&self, item: &AuditEventReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AuditEventReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAuditEventReviewStore {
    pub items: RefCell<BTreeMap<String, AuditEventReview>>,
    pub order: RefCell<Vec<String>>,
}

impl AuditEventReviewStore for MemoryAuditEventReviewStore {
    fn find(&self, code: &str) -> Option<AuditEventReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AuditEventReview) {
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

    fn list(&self) -> Vec<AuditEventReview> {
        let mut out: Vec<AuditEventReview> = Vec::new();
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

pub fn new_memory_audit_event_review_store() -> Rc<dyn AuditEventReviewStore> {
    Rc::new(MemoryAuditEventReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAuditEventReviewStore {
    pub inner: Rc<dyn AuditEventReviewStore>,
    pub cache: RefCell<cache::TtlCache<AuditEventReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AuditEventReviewStore for CachedAuditEventReviewStore {
    fn find(&self, code: &str) -> Option<AuditEventReview> {
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

    fn save(&self, item: &AuditEventReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AuditEventReview> {
        self.inner.list()
    }
}

pub fn new_cached_audit_event_review_store(
    inner: Rc<dyn AuditEventReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AuditEventReviewStore> {
    Rc::new(CachedAuditEventReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AuditEventReview>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects audit event reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventReviewQuery {
    pub segment: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AuditEventReviewQuery {
    pub fn matches(&self, item: &AuditEventReview) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
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

    pub fn apply(&self, items: &[AuditEventReview]) -> Vec<AuditEventReview> {
        let matched = collections::filter(items, |candidate: &AuditEventReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_audit_event_reviews(
    store: Rc<dyn AuditEventReviewStore>,
    query: &AuditEventReviewQuery,
) -> Vec<AuditEventReview> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AuditEventReviewStore> = new_memory_audit_event_review_store();
        let item = new_audit_event_review("code-616", "name-672");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
