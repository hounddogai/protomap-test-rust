use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global forecast rollups by code.
pub trait GlobalForecastRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalForecastRollup>;
    fn save(&self, item: &GlobalForecastRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalForecastRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalForecastRollupStore {
    pub items: RefCell<BTreeMap<String, GlobalForecastRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalForecastRollupStore for MemoryGlobalForecastRollupStore {
    fn find(&self, code: &str) -> Option<GlobalForecastRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalForecastRollup) {
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

    fn list(&self) -> Vec<GlobalForecastRollup> {
        let mut out: Vec<GlobalForecastRollup> = Vec::new();
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

pub fn new_memory_global_forecast_rollup_store() -> Rc<dyn GlobalForecastRollupStore> {
    Rc::new(MemoryGlobalForecastRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalForecastRollupStore {
    pub inner: Rc<dyn GlobalForecastRollupStore>,
    pub cache: RefCell<cache::TtlCache<GlobalForecastRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalForecastRollupStore for CachedGlobalForecastRollupStore {
    fn find(&self, code: &str) -> Option<GlobalForecastRollup> {
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

    fn save(&self, item: &GlobalForecastRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalForecastRollup> {
        self.inner.list()
    }
}

pub fn new_cached_global_forecast_rollup_store(
    inner: Rc<dyn GlobalForecastRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalForecastRollupStore> {
    Rc::new(CachedGlobalForecastRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalForecastRollup>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global forecast rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastRollupQuery {
    pub title: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalForecastRollupQuery {
    pub fn matches(&self, item: &GlobalForecastRollup) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalForecastRollup]) -> Vec<GlobalForecastRollup> {
        let matched = collections::filter(items, |candidate: &GlobalForecastRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_forecast_rollups(
    store: Rc<dyn GlobalForecastRollupStore>,
    query: &GlobalForecastRollupQuery,
) -> Vec<GlobalForecastRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalForecastRollupStore> = new_memory_global_forecast_rollup_store();
        let item = new_global_forecast_rollup("code-826", "title-298");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
