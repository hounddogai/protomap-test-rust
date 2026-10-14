use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal usage scores by code.
pub trait SeasonalUsageScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalUsageScore>;
    fn save(&self, item: &SeasonalUsageScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalUsageScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalUsageScoreStore {
    pub items: RefCell<BTreeMap<String, SeasonalUsageScore>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalUsageScoreStore for MemorySeasonalUsageScoreStore {
    fn find(&self, code: &str) -> Option<SeasonalUsageScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalUsageScore) {
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

    fn list(&self) -> Vec<SeasonalUsageScore> {
        let mut out: Vec<SeasonalUsageScore> = Vec::new();
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

pub fn new_memory_seasonal_usage_score_store() -> Rc<dyn SeasonalUsageScoreStore> {
    Rc::new(MemorySeasonalUsageScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalUsageScoreStore {
    pub inner: Rc<dyn SeasonalUsageScoreStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalUsageScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalUsageScoreStore for CachedSeasonalUsageScoreStore {
    fn find(&self, code: &str) -> Option<SeasonalUsageScore> {
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

    fn save(&self, item: &SeasonalUsageScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalUsageScore> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_usage_score_store(
    inner: Rc<dyn SeasonalUsageScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalUsageScoreStore> {
    Rc::new(CachedSeasonalUsageScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalUsageScore>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal usage scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalUsageScoreQuery {
    pub name: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalUsageScoreQuery {
    pub fn matches(&self, item: &SeasonalUsageScore) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalUsageScore]) -> Vec<SeasonalUsageScore> {
        let matched = collections::filter(items, |candidate: &SeasonalUsageScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_usage_scores(
    store: Rc<dyn SeasonalUsageScoreStore>,
    query: &SeasonalUsageScoreQuery,
) -> Vec<SeasonalUsageScore> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalUsageScoreStore> = new_memory_seasonal_usage_score_store();
        let item = new_seasonal_usage_score("code-801", "name-1");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
