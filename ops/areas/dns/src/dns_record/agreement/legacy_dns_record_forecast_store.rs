use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy dns record forecasts by code.
pub trait LegacyDnsRecordForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyDnsRecordForecast>;
    fn save(&self, item: &LegacyDnsRecordForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyDnsRecordForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyDnsRecordForecastStore {
    pub items: RefCell<BTreeMap<String, LegacyDnsRecordForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyDnsRecordForecastStore for MemoryLegacyDnsRecordForecastStore {
    fn find(&self, code: &str) -> Option<LegacyDnsRecordForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyDnsRecordForecast) {
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

    fn list(&self) -> Vec<LegacyDnsRecordForecast> {
        let mut out: Vec<LegacyDnsRecordForecast> = Vec::new();
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

pub fn new_memory_legacy_dns_record_forecast_store() -> Rc<dyn LegacyDnsRecordForecastStore> {
    Rc::new(MemoryLegacyDnsRecordForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyDnsRecordForecastStore {
    pub inner: Rc<dyn LegacyDnsRecordForecastStore>,
    pub cache: RefCell<cache::TtlCache<LegacyDnsRecordForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyDnsRecordForecastStore for CachedLegacyDnsRecordForecastStore {
    fn find(&self, code: &str) -> Option<LegacyDnsRecordForecast> {
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

    fn save(&self, item: &LegacyDnsRecordForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyDnsRecordForecast> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_dns_record_forecast_store(
    inner: Rc<dyn LegacyDnsRecordForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyDnsRecordForecastStore> {
    Rc::new(CachedLegacyDnsRecordForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyDnsRecordForecast>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy dns record forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDnsRecordForecastQuery {
    pub category: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyDnsRecordForecastQuery {
    pub fn matches(&self, item: &LegacyDnsRecordForecast) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyDnsRecordForecast]) -> Vec<LegacyDnsRecordForecast> {
        let matched = collections::filter(items, |candidate: &LegacyDnsRecordForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_dns_record_forecasts(
    store: Rc<dyn LegacyDnsRecordForecastStore>,
    query: &LegacyDnsRecordForecastQuery,
) -> Vec<LegacyDnsRecordForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyDnsRecordForecastStore> = new_memory_legacy_dns_record_forecast_store();
        let item = new_legacy_dns_record_forecast("code-747", "name-533");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
