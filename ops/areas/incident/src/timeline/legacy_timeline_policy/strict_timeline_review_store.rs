use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict timeline reviews by code.
pub trait StrictTimelineReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictTimelineReview>;
    fn save(&self, item: &StrictTimelineReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictTimelineReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictTimelineReviewStore {
    pub items: RefCell<BTreeMap<String, StrictTimelineReview>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictTimelineReviewStore for MemoryStrictTimelineReviewStore {
    fn find(&self, code: &str) -> Option<StrictTimelineReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictTimelineReview) {
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

    fn list(&self) -> Vec<StrictTimelineReview> {
        let mut out: Vec<StrictTimelineReview> = Vec::new();
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

pub fn new_memory_strict_timeline_review_store() -> Rc<dyn StrictTimelineReviewStore> {
    Rc::new(MemoryStrictTimelineReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictTimelineReviewStore {
    pub inner: Rc<dyn StrictTimelineReviewStore>,
    pub cache: RefCell<cache::TtlCache<StrictTimelineReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictTimelineReviewStore for CachedStrictTimelineReviewStore {
    fn find(&self, code: &str) -> Option<StrictTimelineReview> {
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

    fn save(&self, item: &StrictTimelineReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictTimelineReview> {
        self.inner.list()
    }
}

pub fn new_cached_strict_timeline_review_store(
    inner: Rc<dyn StrictTimelineReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictTimelineReviewStore> {
    Rc::new(CachedStrictTimelineReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictTimelineReview>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict timeline reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineReviewQuery {
    pub region: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictTimelineReviewQuery {
    pub fn matches(&self, item: &StrictTimelineReview) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[StrictTimelineReview]) -> Vec<StrictTimelineReview> {
        let matched = collections::filter(items, |candidate: &StrictTimelineReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_timeline_reviews(
    store: Rc<dyn StrictTimelineReviewStore>,
    query: &StrictTimelineReviewQuery,
) -> Vec<StrictTimelineReview> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn StrictTimelineReviewStore> = new_memory_strict_timeline_review_store();
        let item = new_strict_timeline_review("code-58", "title-250");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
