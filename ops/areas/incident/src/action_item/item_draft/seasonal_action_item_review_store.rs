use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal action item reviews by code.
pub trait SeasonalActionItemReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalActionItemReview>;
    fn save(&self, item: &SeasonalActionItemReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalActionItemReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalActionItemReviewStore {
    pub items: RefCell<BTreeMap<String, SeasonalActionItemReview>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalActionItemReviewStore for MemorySeasonalActionItemReviewStore {
    fn find(&self, code: &str) -> Option<SeasonalActionItemReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalActionItemReview) {
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

    fn list(&self) -> Vec<SeasonalActionItemReview> {
        let mut out: Vec<SeasonalActionItemReview> = Vec::new();
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

pub fn new_memory_seasonal_action_item_review_store() -> Rc<dyn SeasonalActionItemReviewStore> {
    Rc::new(MemorySeasonalActionItemReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalActionItemReviewStore {
    pub inner: Rc<dyn SeasonalActionItemReviewStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalActionItemReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalActionItemReviewStore for CachedSeasonalActionItemReviewStore {
    fn find(&self, code: &str) -> Option<SeasonalActionItemReview> {
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

    fn save(&self, item: &SeasonalActionItemReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalActionItemReview> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_action_item_review_store(
    inner: Rc<dyn SeasonalActionItemReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalActionItemReviewStore> {
    Rc::new(CachedSeasonalActionItemReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalActionItemReview>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal action item reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemReviewQuery {
    pub label: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalActionItemReviewQuery {
    pub fn matches(&self, item: &SeasonalActionItemReview) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalActionItemReview]) -> Vec<SeasonalActionItemReview> {
        let matched = collections::filter(items, |candidate: &SeasonalActionItemReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_action_item_reviews(
    store: Rc<dyn SeasonalActionItemReviewStore>,
    query: &SeasonalActionItemReviewQuery,
) -> Vec<SeasonalActionItemReview> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalActionItemReviewStore> = new_memory_seasonal_action_item_review_store();
        let item = new_seasonal_action_item_review("code-187", "label-445");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
