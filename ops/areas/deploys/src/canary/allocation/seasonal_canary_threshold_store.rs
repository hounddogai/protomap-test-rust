use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal canary thresholds by code.
pub trait SeasonalCanaryThresholdStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalCanaryThreshold>;
    fn save(&self, item: &SeasonalCanaryThreshold);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalCanaryThreshold>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalCanaryThresholdStore {
    pub items: RefCell<BTreeMap<String, SeasonalCanaryThreshold>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalCanaryThresholdStore for MemorySeasonalCanaryThresholdStore {
    fn find(&self, code: &str) -> Option<SeasonalCanaryThreshold> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalCanaryThreshold) {
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

    fn list(&self) -> Vec<SeasonalCanaryThreshold> {
        let mut out: Vec<SeasonalCanaryThreshold> = Vec::new();
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

pub fn new_memory_seasonal_canary_threshold_store() -> Rc<dyn SeasonalCanaryThresholdStore> {
    Rc::new(MemorySeasonalCanaryThresholdStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalCanaryThresholdStore {
    pub inner: Rc<dyn SeasonalCanaryThresholdStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalCanaryThreshold>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalCanaryThresholdStore for CachedSeasonalCanaryThresholdStore {
    fn find(&self, code: &str) -> Option<SeasonalCanaryThreshold> {
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

    fn save(&self, item: &SeasonalCanaryThreshold) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalCanaryThreshold> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_canary_threshold_store(
    inner: Rc<dyn SeasonalCanaryThresholdStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalCanaryThresholdStore> {
    Rc::new(CachedSeasonalCanaryThresholdStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalCanaryThreshold>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal canary thresholds. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryThresholdQuery {
    pub channel: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalCanaryThresholdQuery {
    pub fn matches(&self, item: &SeasonalCanaryThreshold) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalCanaryThreshold]) -> Vec<SeasonalCanaryThreshold> {
        let matched = collections::filter(items, |candidate: &SeasonalCanaryThreshold| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_canary_thresholds(
    store: Rc<dyn SeasonalCanaryThresholdStore>,
    query: &SeasonalCanaryThresholdQuery,
) -> Vec<SeasonalCanaryThreshold> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalCanaryThresholdStore> = new_memory_seasonal_canary_threshold_store();
        let item = new_seasonal_canary_threshold("code-326", "name-636");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
