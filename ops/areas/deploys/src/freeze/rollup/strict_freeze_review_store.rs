use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict freeze reviews by code.
pub trait StrictFreezeReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictFreezeReview>;
    fn save(&self, item: &StrictFreezeReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictFreezeReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictFreezeReviewStore {
    pub items: RefCell<BTreeMap<String, StrictFreezeReview>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictFreezeReviewStore for MemoryStrictFreezeReviewStore {
    fn find(&self, code: &str) -> Option<StrictFreezeReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictFreezeReview) {
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

    fn list(&self) -> Vec<StrictFreezeReview> {
        let mut out: Vec<StrictFreezeReview> = Vec::new();
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

pub fn new_memory_strict_freeze_review_store() -> Rc<dyn StrictFreezeReviewStore> {
    Rc::new(MemoryStrictFreezeReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictFreezeReviewStore {
    pub inner: Rc<dyn StrictFreezeReviewStore>,
    pub cache: RefCell<cache::TtlCache<StrictFreezeReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictFreezeReviewStore for CachedStrictFreezeReviewStore {
    fn find(&self, code: &str) -> Option<StrictFreezeReview> {
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

    fn save(&self, item: &StrictFreezeReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictFreezeReview> {
        self.inner.list()
    }
}

pub fn new_cached_strict_freeze_review_store(
    inner: Rc<dyn StrictFreezeReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictFreezeReviewStore> {
    Rc::new(CachedStrictFreezeReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictFreezeReview>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict freeze reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFreezeReviewQuery {
    pub name: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictFreezeReviewQuery {
    pub fn matches(&self, item: &StrictFreezeReview) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictFreezeReview]) -> Vec<StrictFreezeReview> {
        let matched = collections::filter(items, |candidate: &StrictFreezeReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_freeze_reviews(
    store: Rc<dyn StrictFreezeReviewStore>,
    query: &StrictFreezeReviewQuery,
) -> Vec<StrictFreezeReview> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn StrictFreezeReviewStore> = new_memory_strict_freeze_review_store();
        let item = new_strict_freeze_review("code-642", "name-122");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
