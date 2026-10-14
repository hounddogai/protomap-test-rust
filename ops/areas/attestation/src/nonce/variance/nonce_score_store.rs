use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps nonce scores by code.
pub trait NonceScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<NonceScore>;
    fn save(&self, item: &NonceScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<NonceScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryNonceScoreStore {
    pub items: RefCell<BTreeMap<String, NonceScore>>,
    pub order: RefCell<Vec<String>>,
}

impl NonceScoreStore for MemoryNonceScoreStore {
    fn find(&self, code: &str) -> Option<NonceScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &NonceScore) {
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

    fn list(&self) -> Vec<NonceScore> {
        let mut out: Vec<NonceScore> = Vec::new();
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

pub fn new_memory_nonce_score_store() -> Rc<dyn NonceScoreStore> {
    Rc::new(MemoryNonceScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedNonceScoreStore {
    pub inner: Rc<dyn NonceScoreStore>,
    pub cache: RefCell<cache::TtlCache<NonceScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl NonceScoreStore for CachedNonceScoreStore {
    fn find(&self, code: &str) -> Option<NonceScore> {
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

    fn save(&self, item: &NonceScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<NonceScore> {
        self.inner.list()
    }
}

pub fn new_cached_nonce_score_store(
    inner: Rc<dyn NonceScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn NonceScoreStore> {
    Rc::new(CachedNonceScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<NonceScore>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects nonce scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceScoreQuery {
    pub name: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl NonceScoreQuery {
    pub fn matches(&self, item: &NonceScore) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[NonceScore]) -> Vec<NonceScore> {
        let matched = collections::filter(items, |candidate: &NonceScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_nonce_scores(store: Rc<dyn NonceScoreStore>, query: &NonceScoreQuery) -> Vec<NonceScore> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn NonceScoreStore> = new_memory_nonce_score_store();
        let item = new_nonce_score("code-753", "name-390");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
