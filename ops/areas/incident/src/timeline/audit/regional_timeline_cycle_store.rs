use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional timeline cycles by code.
pub trait RegionalTimelineCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalTimelineCycle>;
    fn save(&self, item: &RegionalTimelineCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalTimelineCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalTimelineCycleStore {
    pub items: RefCell<BTreeMap<String, RegionalTimelineCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalTimelineCycleStore for MemoryRegionalTimelineCycleStore {
    fn find(&self, code: &str) -> Option<RegionalTimelineCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalTimelineCycle) {
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

    fn list(&self) -> Vec<RegionalTimelineCycle> {
        let mut out: Vec<RegionalTimelineCycle> = Vec::new();
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

pub fn new_memory_regional_timeline_cycle_store() -> Rc<dyn RegionalTimelineCycleStore> {
    Rc::new(MemoryRegionalTimelineCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalTimelineCycleStore {
    pub inner: Rc<dyn RegionalTimelineCycleStore>,
    pub cache: RefCell<cache::TtlCache<RegionalTimelineCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalTimelineCycleStore for CachedRegionalTimelineCycleStore {
    fn find(&self, code: &str) -> Option<RegionalTimelineCycle> {
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

    fn save(&self, item: &RegionalTimelineCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalTimelineCycle> {
        self.inner.list()
    }
}

pub fn new_cached_regional_timeline_cycle_store(
    inner: Rc<dyn RegionalTimelineCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalTimelineCycleStore> {
    Rc::new(CachedRegionalTimelineCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalTimelineCycle>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional timeline cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineCycleQuery {
    pub region: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalTimelineCycleQuery {
    pub fn matches(&self, item: &RegionalTimelineCycle) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalTimelineCycle]) -> Vec<RegionalTimelineCycle> {
        let matched = collections::filter(items, |candidate: &RegionalTimelineCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_timeline_cycles(
    store: Rc<dyn RegionalTimelineCycleStore>,
    query: &RegionalTimelineCycleQuery,
) -> Vec<RegionalTimelineCycle> {
    query.apply(&store.list())
}
