use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal forecast groups by code.
pub trait SeasonalForecastGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalForecastGroup>;
    fn save(&self, item: &SeasonalForecastGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalForecastGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalForecastGroupStore {
    pub items: RefCell<BTreeMap<String, SeasonalForecastGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalForecastGroupStore for MemorySeasonalForecastGroupStore {
    fn find(&self, code: &str) -> Option<SeasonalForecastGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalForecastGroup) {
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

    fn list(&self) -> Vec<SeasonalForecastGroup> {
        let mut out: Vec<SeasonalForecastGroup> = Vec::new();
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

pub fn new_memory_seasonal_forecast_group_store() -> Rc<dyn SeasonalForecastGroupStore> {
    Rc::new(MemorySeasonalForecastGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalForecastGroupStore {
    pub inner: Rc<dyn SeasonalForecastGroupStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalForecastGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalForecastGroupStore for CachedSeasonalForecastGroupStore {
    fn find(&self, code: &str) -> Option<SeasonalForecastGroup> {
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

    fn save(&self, item: &SeasonalForecastGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalForecastGroup> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_forecast_group_store(
    inner: Rc<dyn SeasonalForecastGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalForecastGroupStore> {
    Rc::new(CachedSeasonalForecastGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalForecastGroup>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal forecast groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastGroupQuery {
    pub namespace: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalForecastGroupQuery {
    pub fn matches(&self, item: &SeasonalForecastGroup) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalForecastGroup]) -> Vec<SeasonalForecastGroup> {
        let matched = collections::filter(items, |candidate: &SeasonalForecastGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_forecast_groups(
    store: Rc<dyn SeasonalForecastGroupStore>,
    query: &SeasonalForecastGroupQuery,
) -> Vec<SeasonalForecastGroup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalForecastGroupStore> = new_memory_seasonal_forecast_group_store();
        let item = new_seasonal_forecast_group("code-93", "label-421");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
