use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps budget burn reviews by code.
pub trait BudgetBurnReviewStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BudgetBurnReview>;
    fn save(&self, item: &BudgetBurnReview);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BudgetBurnReview>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBudgetBurnReviewStore {
    pub items: RefCell<BTreeMap<String, BudgetBurnReview>>,
    pub order: RefCell<Vec<String>>,
}

impl BudgetBurnReviewStore for MemoryBudgetBurnReviewStore {
    fn find(&self, code: &str) -> Option<BudgetBurnReview> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BudgetBurnReview) {
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

    fn list(&self) -> Vec<BudgetBurnReview> {
        let mut out: Vec<BudgetBurnReview> = Vec::new();
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

pub fn new_memory_budget_burn_review_store() -> Rc<dyn BudgetBurnReviewStore> {
    Rc::new(MemoryBudgetBurnReviewStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBudgetBurnReviewStore {
    pub inner: Rc<dyn BudgetBurnReviewStore>,
    pub cache: RefCell<cache::TtlCache<BudgetBurnReview>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BudgetBurnReviewStore for CachedBudgetBurnReviewStore {
    fn find(&self, code: &str) -> Option<BudgetBurnReview> {
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

    fn save(&self, item: &BudgetBurnReview) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BudgetBurnReview> {
        self.inner.list()
    }
}

pub fn new_cached_budget_burn_review_store(
    inner: Rc<dyn BudgetBurnReviewStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BudgetBurnReviewStore> {
    Rc::new(CachedBudgetBurnReviewStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BudgetBurnReview>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects budget burn reviews. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnReviewQuery {
    pub locale: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BudgetBurnReviewQuery {
    pub fn matches(&self, item: &BudgetBurnReview) -> bool {
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

    pub fn apply(&self, items: &[BudgetBurnReview]) -> Vec<BudgetBurnReview> {
        let matched = collections::filter(items, |candidate: &BudgetBurnReview| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_budget_burn_reviews(
    store: Rc<dyn BudgetBurnReviewStore>,
    query: &BudgetBurnReviewQuery,
) -> Vec<BudgetBurnReview> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BudgetBurnReviewStore> = new_memory_budget_burn_review_store();
        let item = new_budget_burn_review("code-454", "title-984");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
