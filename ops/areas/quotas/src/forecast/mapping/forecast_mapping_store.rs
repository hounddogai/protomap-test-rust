use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps forecast mappings by code.
pub trait ForecastMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ForecastMapping>;
    fn save(&self, item: &ForecastMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ForecastMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryForecastMappingStore {
    pub items: RefCell<BTreeMap<String, ForecastMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl ForecastMappingStore for MemoryForecastMappingStore {
    fn find(&self, code: &str) -> Option<ForecastMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ForecastMapping) {
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

    fn list(&self) -> Vec<ForecastMapping> {
        let mut out: Vec<ForecastMapping> = Vec::new();
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

pub fn new_memory_forecast_mapping_store() -> Rc<dyn ForecastMappingStore> {
    Rc::new(MemoryForecastMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedForecastMappingStore {
    pub inner: Rc<dyn ForecastMappingStore>,
    pub cache: RefCell<cache::TtlCache<ForecastMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ForecastMappingStore for CachedForecastMappingStore {
    fn find(&self, code: &str) -> Option<ForecastMapping> {
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

    fn save(&self, item: &ForecastMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ForecastMapping> {
        self.inner.list()
    }
}

pub fn new_cached_forecast_mapping_store(
    inner: Rc<dyn ForecastMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ForecastMappingStore> {
    Rc::new(CachedForecastMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ForecastMapping>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects forecast mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMappingQuery {
    pub title: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ForecastMappingQuery {
    pub fn matches(&self, item: &ForecastMapping) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[ForecastMapping]) -> Vec<ForecastMapping> {
        let matched = collections::filter(items, |candidate: &ForecastMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_forecast_mappings(
    store: Rc<dyn ForecastMappingStore>,
    query: &ForecastMappingQuery,
) -> Vec<ForecastMapping> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ForecastMappingStore> = new_memory_forecast_mapping_store();
        let item = new_forecast_mapping("code-928", "title-614");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
