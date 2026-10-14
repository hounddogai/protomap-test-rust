use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps zone estimates by code.
pub trait ZoneEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ZoneEstimate>;
    fn save(&self, item: &ZoneEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ZoneEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryZoneEstimateStore {
    pub items: RefCell<BTreeMap<String, ZoneEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl ZoneEstimateStore for MemoryZoneEstimateStore {
    fn find(&self, code: &str) -> Option<ZoneEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ZoneEstimate) {
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

    fn list(&self) -> Vec<ZoneEstimate> {
        let mut out: Vec<ZoneEstimate> = Vec::new();
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

pub fn new_memory_zone_estimate_store() -> Rc<dyn ZoneEstimateStore> {
    Rc::new(MemoryZoneEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedZoneEstimateStore {
    pub inner: Rc<dyn ZoneEstimateStore>,
    pub cache: RefCell<cache::TtlCache<ZoneEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ZoneEstimateStore for CachedZoneEstimateStore {
    fn find(&self, code: &str) -> Option<ZoneEstimate> {
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

    fn save(&self, item: &ZoneEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ZoneEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_zone_estimate_store(
    inner: Rc<dyn ZoneEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ZoneEstimateStore> {
    Rc::new(CachedZoneEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ZoneEstimate>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects zone estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneEstimateQuery {
    pub title: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ZoneEstimateQuery {
    pub fn matches(&self, item: &ZoneEstimate) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ZoneEstimate]) -> Vec<ZoneEstimate> {
        let matched = collections::filter(items, |candidate: &ZoneEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_zone_estimates(store: Rc<dyn ZoneEstimateStore>, query: &ZoneEstimateQuery) -> Vec<ZoneEstimate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ZoneEstimateStore> = new_memory_zone_estimate_store();
        let item = new_zone_estimate("code-329", "title-215");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
