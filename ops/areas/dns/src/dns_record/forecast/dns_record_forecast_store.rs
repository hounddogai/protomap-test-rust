use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dns record forecasts by code.
pub trait DnsRecordForecastStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DnsRecordForecast>;
    fn save(&self, item: &DnsRecordForecast);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DnsRecordForecast>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDnsRecordForecastStore {
    pub items: RefCell<BTreeMap<String, DnsRecordForecast>>,
    pub order: RefCell<Vec<String>>,
}

impl DnsRecordForecastStore for MemoryDnsRecordForecastStore {
    fn find(&self, code: &str) -> Option<DnsRecordForecast> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DnsRecordForecast) {
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

    fn list(&self) -> Vec<DnsRecordForecast> {
        let mut out: Vec<DnsRecordForecast> = Vec::new();
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

pub fn new_memory_dns_record_forecast_store() -> Rc<dyn DnsRecordForecastStore> {
    Rc::new(MemoryDnsRecordForecastStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDnsRecordForecastStore {
    pub inner: Rc<dyn DnsRecordForecastStore>,
    pub cache: RefCell<cache::TtlCache<DnsRecordForecast>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DnsRecordForecastStore for CachedDnsRecordForecastStore {
    fn find(&self, code: &str) -> Option<DnsRecordForecast> {
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

    fn save(&self, item: &DnsRecordForecast) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DnsRecordForecast> {
        self.inner.list()
    }
}

pub fn new_cached_dns_record_forecast_store(
    inner: Rc<dyn DnsRecordForecastStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DnsRecordForecastStore> {
    Rc::new(CachedDnsRecordForecastStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DnsRecordForecast>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dns record forecasts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordForecastQuery {
    pub name: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DnsRecordForecastQuery {
    pub fn matches(&self, item: &DnsRecordForecast) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[DnsRecordForecast]) -> Vec<DnsRecordForecast> {
        let matched = collections::filter(items, |candidate: &DnsRecordForecast| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dns_record_forecasts(
    store: Rc<dyn DnsRecordForecastStore>,
    query: &DnsRecordForecastQuery,
) -> Vec<DnsRecordForecast> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DnsRecordForecastStore> = new_memory_dns_record_forecast_store();
        let item = new_dns_record_forecast("code-276", "name-571");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
