use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling verifier scores by code.
pub trait RollingVerifierScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingVerifierScore>;
    fn save(&self, item: &RollingVerifierScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingVerifierScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingVerifierScoreStore {
    pub items: RefCell<BTreeMap<String, RollingVerifierScore>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingVerifierScoreStore for MemoryRollingVerifierScoreStore {
    fn find(&self, code: &str) -> Option<RollingVerifierScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingVerifierScore) {
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

    fn list(&self) -> Vec<RollingVerifierScore> {
        let mut out: Vec<RollingVerifierScore> = Vec::new();
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

pub fn new_memory_rolling_verifier_score_store() -> Rc<dyn RollingVerifierScoreStore> {
    Rc::new(MemoryRollingVerifierScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingVerifierScoreStore {
    pub inner: Rc<dyn RollingVerifierScoreStore>,
    pub cache: RefCell<cache::TtlCache<RollingVerifierScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingVerifierScoreStore for CachedRollingVerifierScoreStore {
    fn find(&self, code: &str) -> Option<RollingVerifierScore> {
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

    fn save(&self, item: &RollingVerifierScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingVerifierScore> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_verifier_score_store(
    inner: Rc<dyn RollingVerifierScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingVerifierScoreStore> {
    Rc::new(CachedRollingVerifierScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingVerifierScore>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling verifier scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierScoreQuery {
    pub cluster: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingVerifierScoreQuery {
    pub fn matches(&self, item: &RollingVerifierScore) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingVerifierScore]) -> Vec<RollingVerifierScore> {
        let matched = collections::filter(items, |candidate: &RollingVerifierScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_verifier_scores(
    store: Rc<dyn RollingVerifierScoreStore>,
    query: &RollingVerifierScoreQuery,
) -> Vec<RollingVerifierScore> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingVerifierScoreStore> = new_memory_rolling_verifier_score_store();
        let item = new_rolling_verifier_score("code-808", "name-506");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
