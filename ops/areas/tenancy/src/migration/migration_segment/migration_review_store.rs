use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps migration reviews by code.
pub trait MigrationReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MigrationReview>;
    fn save(&self, item: &MigrationReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MigrationReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMigrationReviewStore {
    pub items: RefCell<BTreeMap<String, MigrationReview>>,
    pub order: RefCell<Vec<String>>,
}

impl MigrationReviewStore for MemoryMigrationReviewStore {
    fn find(&self, code: &str) -> Option<MigrationReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MigrationReview) {
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

    fn list(&self) -> Vec<MigrationReview> {
        let mut out: Vec<MigrationReview> = Vec::new();
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

pub fn new_memory_migration_review_store() -> Rc<dyn MigrationReviewStore> {
    Rc::new(MemoryMigrationReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMigrationReviewStore {
    pub inner: Rc<dyn MigrationReviewStore>,
    pub cache: RefCell<cache::TtlCache<MigrationReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MigrationReviewStore for CachedMigrationReviewStore {
    fn find(&self, code: &str) -> Option<MigrationReview> {
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

    fn save(&self, item: &MigrationReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MigrationReview> {
        self.inner.list()
    }
}

pub fn new_cached_migration_review_store(
    inner: Rc<dyn MigrationReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MigrationReviewStore> {
    Rc::new(CachedMigrationReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MigrationReview>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects migration reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationReviewQuery {
    pub channel: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MigrationReviewQuery {
    pub fn matches(&self, item: &MigrationReview) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
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

    pub fn apply(&self, items: &[MigrationReview]) -> Vec<MigrationReview> {
        let matched = collections::filter(items, |candidate: &MigrationReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_migration_reviews(
    store: Rc<dyn MigrationReviewStore>,
    query: &MigrationReviewQuery,
) -> Vec<MigrationReview> {
    query.apply(&store.list())
}
