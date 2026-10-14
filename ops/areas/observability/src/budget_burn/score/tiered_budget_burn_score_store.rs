use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered budget burn scores by code.
pub trait TieredBudgetBurnScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredBudgetBurnScore>;
    fn save(&self, item: &TieredBudgetBurnScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredBudgetBurnScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredBudgetBurnScoreStore {
    pub items: RefCell<BTreeMap<String, TieredBudgetBurnScore>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredBudgetBurnScoreStore for MemoryTieredBudgetBurnScoreStore {
    fn find(&self, code: &str) -> Option<TieredBudgetBurnScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredBudgetBurnScore) {
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

    fn list(&self) -> Vec<TieredBudgetBurnScore> {
        let mut out: Vec<TieredBudgetBurnScore> = Vec::new();
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

pub fn new_memory_tiered_budget_burn_score_store() -> Rc<dyn TieredBudgetBurnScoreStore> {
    Rc::new(MemoryTieredBudgetBurnScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredBudgetBurnScoreStore {
    pub inner: Rc<dyn TieredBudgetBurnScoreStore>,
    pub cache: RefCell<cache::TtlCache<TieredBudgetBurnScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredBudgetBurnScoreStore for CachedTieredBudgetBurnScoreStore {
    fn find(&self, code: &str) -> Option<TieredBudgetBurnScore> {
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

    fn save(&self, item: &TieredBudgetBurnScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredBudgetBurnScore> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_budget_burn_score_store(
    inner: Rc<dyn TieredBudgetBurnScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredBudgetBurnScoreStore> {
    Rc::new(CachedTieredBudgetBurnScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredBudgetBurnScore>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered budget burn scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnScoreQuery {
    pub namespace: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredBudgetBurnScoreQuery {
    pub fn matches(&self, item: &TieredBudgetBurnScore) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.attempts < self.min_attempts {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredBudgetBurnScore]) -> Vec<TieredBudgetBurnScore> {
        let matched = collections::filter(items, |candidate: &TieredBudgetBurnScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_budget_burn_scores(
    store: Rc<dyn TieredBudgetBurnScoreStore>,
    query: &TieredBudgetBurnScoreQuery,
) -> Vec<TieredBudgetBurnScore> {
    query.apply(&store.list())
}
