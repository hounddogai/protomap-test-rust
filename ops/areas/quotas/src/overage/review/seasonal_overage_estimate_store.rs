use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal overage estimates by code.
pub trait SeasonalOverageEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalOverageEstimate>;
    fn save(&self, item: &SeasonalOverageEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalOverageEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalOverageEstimateStore {
    pub items: RefCell<BTreeMap<String, SeasonalOverageEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalOverageEstimateStore for MemorySeasonalOverageEstimateStore {
    fn find(&self, code: &str) -> Option<SeasonalOverageEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalOverageEstimate) {
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

    fn list(&self) -> Vec<SeasonalOverageEstimate> {
        let mut out: Vec<SeasonalOverageEstimate> = Vec::new();
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

pub fn new_memory_seasonal_overage_estimate_store() -> Rc<dyn SeasonalOverageEstimateStore> {
    Rc::new(MemorySeasonalOverageEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalOverageEstimateStore {
    pub inner: Rc<dyn SeasonalOverageEstimateStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalOverageEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalOverageEstimateStore for CachedSeasonalOverageEstimateStore {
    fn find(&self, code: &str) -> Option<SeasonalOverageEstimate> {
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

    fn save(&self, item: &SeasonalOverageEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalOverageEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_overage_estimate_store(
    inner: Rc<dyn SeasonalOverageEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalOverageEstimateStore> {
    Rc::new(CachedSeasonalOverageEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalOverageEstimate>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal overage estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalOverageEstimateQuery {
    pub scope: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalOverageEstimateQuery {
    pub fn matches(&self, item: &SeasonalOverageEstimate) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalOverageEstimate]) -> Vec<SeasonalOverageEstimate> {
        let matched = collections::filter(items, |candidate: &SeasonalOverageEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_overage_estimates(
    store: Rc<dyn SeasonalOverageEstimateStore>,
    query: &SeasonalOverageEstimateQuery,
) -> Vec<SeasonalOverageEstimate> {
    query.apply(&store.list())
}
