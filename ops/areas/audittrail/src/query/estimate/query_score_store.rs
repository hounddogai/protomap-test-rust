use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps query scores by code.
pub trait QueryScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QueryScore>;
    fn save(&self, item: &QueryScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QueryScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQueryScoreStore {
    pub items: RefCell<BTreeMap<String, QueryScore>>,
    pub order: RefCell<Vec<String>>,
}

impl QueryScoreStore for MemoryQueryScoreStore {
    fn find(&self, code: &str) -> Option<QueryScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QueryScore) {
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

    fn list(&self) -> Vec<QueryScore> {
        let mut out: Vec<QueryScore> = Vec::new();
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

pub fn new_memory_query_score_store() -> Rc<dyn QueryScoreStore> {
    Rc::new(MemoryQueryScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQueryScoreStore {
    pub inner: Rc<dyn QueryScoreStore>,
    pub cache: RefCell<cache::TtlCache<QueryScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QueryScoreStore for CachedQueryScoreStore {
    fn find(&self, code: &str) -> Option<QueryScore> {
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

    fn save(&self, item: &QueryScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QueryScore> {
        self.inner.list()
    }
}

pub fn new_cached_query_score_store(
    inner: Rc<dyn QueryScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QueryScoreStore> {
    Rc::new(CachedQueryScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QueryScore>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects query scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryScoreQuery {
    pub label: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QueryScoreQuery {
    pub fn matches(&self, item: &QueryScore) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[QueryScore]) -> Vec<QueryScore> {
        let matched = collections::filter(items, |candidate: &QueryScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_query_scores(store: Rc<dyn QueryScoreStore>, query: &QueryScoreQuery) -> Vec<QueryScore> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn QueryScoreStore> = new_memory_query_score_store();
        let item = new_query_score("code-969", "label-970");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
