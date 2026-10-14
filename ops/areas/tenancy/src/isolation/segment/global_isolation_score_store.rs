use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global isolation scores by code.
pub trait GlobalIsolationScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalIsolationScore>;
    fn save(&self, item: &GlobalIsolationScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalIsolationScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalIsolationScoreStore {
    pub items: RefCell<BTreeMap<String, GlobalIsolationScore>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalIsolationScoreStore for MemoryGlobalIsolationScoreStore {
    fn find(&self, code: &str) -> Option<GlobalIsolationScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalIsolationScore) {
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

    fn list(&self) -> Vec<GlobalIsolationScore> {
        let mut out: Vec<GlobalIsolationScore> = Vec::new();
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

pub fn new_memory_global_isolation_score_store() -> Rc<dyn GlobalIsolationScoreStore> {
    Rc::new(MemoryGlobalIsolationScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalIsolationScoreStore {
    pub inner: Rc<dyn GlobalIsolationScoreStore>,
    pub cache: RefCell<cache::TtlCache<GlobalIsolationScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalIsolationScoreStore for CachedGlobalIsolationScoreStore {
    fn find(&self, code: &str) -> Option<GlobalIsolationScore> {
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

    fn save(&self, item: &GlobalIsolationScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalIsolationScore> {
        self.inner.list()
    }
}

pub fn new_cached_global_isolation_score_store(
    inner: Rc<dyn GlobalIsolationScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalIsolationScoreStore> {
    Rc::new(CachedGlobalIsolationScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalIsolationScore>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global isolation scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIsolationScoreQuery {
    pub channel: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalIsolationScoreQuery {
    pub fn matches(&self, item: &GlobalIsolationScore) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalIsolationScore]) -> Vec<GlobalIsolationScore> {
        let matched = collections::filter(items, |candidate: &GlobalIsolationScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_isolation_scores(
    store: Rc<dyn GlobalIsolationScoreStore>,
    query: &GlobalIsolationScoreQuery,
) -> Vec<GlobalIsolationScore> {
    query.apply(&store.list())
}
