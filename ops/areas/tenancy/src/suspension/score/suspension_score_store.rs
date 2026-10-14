use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps suspension scores by code.
pub trait SuspensionScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SuspensionScore>;
    fn save(&self, item: &SuspensionScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SuspensionScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySuspensionScoreStore {
    pub items: RefCell<BTreeMap<String, SuspensionScore>>,
    pub order: RefCell<Vec<String>>,
}

impl SuspensionScoreStore for MemorySuspensionScoreStore {
    fn find(&self, code: &str) -> Option<SuspensionScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SuspensionScore) {
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

    fn list(&self) -> Vec<SuspensionScore> {
        let mut out: Vec<SuspensionScore> = Vec::new();
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

pub fn new_memory_suspension_score_store() -> Rc<dyn SuspensionScoreStore> {
    Rc::new(MemorySuspensionScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSuspensionScoreStore {
    pub inner: Rc<dyn SuspensionScoreStore>,
    pub cache: RefCell<cache::TtlCache<SuspensionScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SuspensionScoreStore for CachedSuspensionScoreStore {
    fn find(&self, code: &str) -> Option<SuspensionScore> {
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

    fn save(&self, item: &SuspensionScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SuspensionScore> {
        self.inner.list()
    }
}

pub fn new_cached_suspension_score_store(
    inner: Rc<dyn SuspensionScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SuspensionScoreStore> {
    Rc::new(CachedSuspensionScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SuspensionScore>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects suspension scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionScoreQuery {
    pub scope: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SuspensionScoreQuery {
    pub fn matches(&self, item: &SuspensionScore) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SuspensionScore]) -> Vec<SuspensionScore> {
        let matched = collections::filter(items, |candidate: &SuspensionScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_suspension_scores(
    store: Rc<dyn SuspensionScoreStore>,
    query: &SuspensionScoreQuery,
) -> Vec<SuspensionScore> {
    query.apply(&store.list())
}
