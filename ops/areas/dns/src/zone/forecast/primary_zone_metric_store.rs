use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary zone metrics by code.
pub trait PrimaryZoneMetricStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryZoneMetric>;
    fn save(&self, item: &PrimaryZoneMetric);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryZoneMetric>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryZoneMetricStore {
    pub items: RefCell<BTreeMap<String, PrimaryZoneMetric>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryZoneMetricStore for MemoryPrimaryZoneMetricStore {
    fn find(&self, code: &str) -> Option<PrimaryZoneMetric> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryZoneMetric) {
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

    fn list(&self) -> Vec<PrimaryZoneMetric> {
        let mut out: Vec<PrimaryZoneMetric> = Vec::new();
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

pub fn new_memory_primary_zone_metric_store() -> Rc<dyn PrimaryZoneMetricStore> {
    Rc::new(MemoryPrimaryZoneMetricStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryZoneMetricStore {
    pub inner: Rc<dyn PrimaryZoneMetricStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryZoneMetric>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryZoneMetricStore for CachedPrimaryZoneMetricStore {
    fn find(&self, code: &str) -> Option<PrimaryZoneMetric> {
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

    fn save(&self, item: &PrimaryZoneMetric) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryZoneMetric> {
        self.inner.list()
    }
}

pub fn new_cached_primary_zone_metric_store(
    inner: Rc<dyn PrimaryZoneMetricStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryZoneMetricStore> {
    Rc::new(CachedPrimaryZoneMetricStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryZoneMetric>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary zone metrics. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryZoneMetricQuery {
    pub unit: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryZoneMetricQuery {
    pub fn matches(&self, item: &PrimaryZoneMetric) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryZoneMetric]) -> Vec<PrimaryZoneMetric> {
        let matched = collections::filter(items, |candidate: &PrimaryZoneMetric| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_zone_metrics(
    store: Rc<dyn PrimaryZoneMetricStore>,
    query: &PrimaryZoneMetricQuery,
) -> Vec<PrimaryZoneMetric> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryZoneMetricStore> = new_memory_primary_zone_metric_store();
        let item = new_primary_zone_metric("code-722", "name-104");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
