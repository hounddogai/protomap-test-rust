use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps limit scores by code.
pub trait LimitScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LimitScore>;
    fn save(&self, item: &LimitScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LimitScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLimitScoreStore {
    pub items: RefCell<BTreeMap<String, LimitScore>>,
    pub order: RefCell<Vec<String>>,
}

impl LimitScoreStore for MemoryLimitScoreStore {
    fn find(&self, code: &str) -> Option<LimitScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LimitScore) {
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

    fn list(&self) -> Vec<LimitScore> {
        let mut out: Vec<LimitScore> = Vec::new();
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

pub fn new_memory_limit_score_store() -> Rc<dyn LimitScoreStore> {
    Rc::new(MemoryLimitScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLimitScoreStore {
    pub inner: Rc<dyn LimitScoreStore>,
    pub cache: RefCell<cache::TtlCache<LimitScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LimitScoreStore for CachedLimitScoreStore {
    fn find(&self, code: &str) -> Option<LimitScore> {
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

    fn save(&self, item: &LimitScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LimitScore> {
        self.inner.list()
    }
}

pub fn new_cached_limit_score_store(
    inner: Rc<dyn LimitScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LimitScoreStore> {
    Rc::new(CachedLimitScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LimitScore>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects limit scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitScoreQuery {
    pub segment: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LimitScoreQuery {
    pub fn matches(&self, item: &LimitScore) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LimitScore]) -> Vec<LimitScore> {
        let matched = collections::filter(items, |candidate: &LimitScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_limit_scores(store: Rc<dyn LimitScoreStore>, query: &LimitScoreQuery) -> Vec<LimitScore> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LimitScoreStore> = new_memory_limit_score_store();
        let item = new_limit_score("code-998", "label-419");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
