use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted page scores by code.
pub trait WeightedPageScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedPageScore>;
    fn save(&self, item: &WeightedPageScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedPageScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedPageScoreStore {
    pub items: RefCell<BTreeMap<String, WeightedPageScore>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedPageScoreStore for MemoryWeightedPageScoreStore {
    fn find(&self, code: &str) -> Option<WeightedPageScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedPageScore) {
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

    fn list(&self) -> Vec<WeightedPageScore> {
        let mut out: Vec<WeightedPageScore> = Vec::new();
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

pub fn new_memory_weighted_page_score_store() -> Rc<dyn WeightedPageScoreStore> {
    Rc::new(MemoryWeightedPageScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedPageScoreStore {
    pub inner: Rc<dyn WeightedPageScoreStore>,
    pub cache: RefCell<cache::TtlCache<WeightedPageScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedPageScoreStore for CachedWeightedPageScoreStore {
    fn find(&self, code: &str) -> Option<WeightedPageScore> {
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

    fn save(&self, item: &WeightedPageScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedPageScore> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_page_score_store(
    inner: Rc<dyn WeightedPageScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedPageScoreStore> {
    Rc::new(CachedWeightedPageScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedPageScore>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted page scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPageScoreQuery {
    pub name: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedPageScoreQuery {
    pub fn matches(&self, item: &WeightedPageScore) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedPageScore]) -> Vec<WeightedPageScore> {
        let matched = collections::filter(items, |candidate: &WeightedPageScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_page_scores(
    store: Rc<dyn WeightedPageScoreStore>,
    query: &WeightedPageScoreQuery,
) -> Vec<WeightedPageScore> {
    query.apply(&store.list())
}
