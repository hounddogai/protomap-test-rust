use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal query variances by code.
pub trait SeasonalQueryVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalQueryVariance>;
    fn save(&self, item: &SeasonalQueryVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalQueryVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalQueryVarianceStore {
    pub items: RefCell<BTreeMap<String, SeasonalQueryVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalQueryVarianceStore for MemorySeasonalQueryVarianceStore {
    fn find(&self, code: &str) -> Option<SeasonalQueryVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalQueryVariance) {
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

    fn list(&self) -> Vec<SeasonalQueryVariance> {
        let mut out: Vec<SeasonalQueryVariance> = Vec::new();
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

pub fn new_memory_seasonal_query_variance_store() -> Rc<dyn SeasonalQueryVarianceStore> {
    Rc::new(MemorySeasonalQueryVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalQueryVarianceStore {
    pub inner: Rc<dyn SeasonalQueryVarianceStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalQueryVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalQueryVarianceStore for CachedSeasonalQueryVarianceStore {
    fn find(&self, code: &str) -> Option<SeasonalQueryVariance> {
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

    fn save(&self, item: &SeasonalQueryVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalQueryVariance> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_query_variance_store(
    inner: Rc<dyn SeasonalQueryVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalQueryVarianceStore> {
    Rc::new(CachedSeasonalQueryVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalQueryVariance>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal query variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQueryVarianceQuery {
    pub cluster: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalQueryVarianceQuery {
    pub fn matches(&self, item: &SeasonalQueryVariance) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[SeasonalQueryVariance]) -> Vec<SeasonalQueryVariance> {
        let matched = collections::filter(items, |candidate: &SeasonalQueryVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_query_variances(
    store: Rc<dyn SeasonalQueryVarianceStore>,
    query: &SeasonalQueryVarianceQuery,
) -> Vec<SeasonalQueryVariance> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalQueryVarianceStore> = new_memory_seasonal_query_variance_store();
        let item = new_seasonal_query_variance("code-462", "title-211");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
