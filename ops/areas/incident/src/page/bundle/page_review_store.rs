use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps page reviews by code.
pub trait PageReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PageReview>;
    fn save(&self, item: &PageReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PageReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPageReviewStore {
    pub items: RefCell<BTreeMap<String, PageReview>>,
    pub order: RefCell<Vec<String>>,
}

impl PageReviewStore for MemoryPageReviewStore {
    fn find(&self, code: &str) -> Option<PageReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PageReview) {
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

    fn list(&self) -> Vec<PageReview> {
        let mut out: Vec<PageReview> = Vec::new();
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

pub fn new_memory_page_review_store() -> Rc<dyn PageReviewStore> {
    Rc::new(MemoryPageReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPageReviewStore {
    pub inner: Rc<dyn PageReviewStore>,
    pub cache: RefCell<cache::TtlCache<PageReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PageReviewStore for CachedPageReviewStore {
    fn find(&self, code: &str) -> Option<PageReview> {
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

    fn save(&self, item: &PageReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PageReview> {
        self.inner.list()
    }
}

pub fn new_cached_page_review_store(
    inner: Rc<dyn PageReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PageReviewStore> {
    Rc::new(CachedPageReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PageReview>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects page reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PageReviewQuery {
    pub region: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PageReviewQuery {
    pub fn matches(&self, item: &PageReview) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PageReview]) -> Vec<PageReview> {
        let matched = collections::filter(items, |candidate: &PageReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_page_reviews(store: Rc<dyn PageReviewStore>, query: &PageReviewQuery) -> Vec<PageReview> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PageReviewStore> = new_memory_page_review_store();
        let item = new_page_review("code-678", "name-154");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
