use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal identity cycles by code.
pub trait SeasonalIdentityCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalIdentityCycle>;
    fn save(&self, item: &SeasonalIdentityCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalIdentityCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalIdentityCycleStore {
    pub items: RefCell<BTreeMap<String, SeasonalIdentityCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalIdentityCycleStore for MemorySeasonalIdentityCycleStore {
    fn find(&self, code: &str) -> Option<SeasonalIdentityCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalIdentityCycle) {
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

    fn list(&self) -> Vec<SeasonalIdentityCycle> {
        let mut out: Vec<SeasonalIdentityCycle> = Vec::new();
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

pub fn new_memory_seasonal_identity_cycle_store() -> Rc<dyn SeasonalIdentityCycleStore> {
    Rc::new(MemorySeasonalIdentityCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalIdentityCycleStore {
    pub inner: Rc<dyn SeasonalIdentityCycleStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalIdentityCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalIdentityCycleStore for CachedSeasonalIdentityCycleStore {
    fn find(&self, code: &str) -> Option<SeasonalIdentityCycle> {
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

    fn save(&self, item: &SeasonalIdentityCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalIdentityCycle> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_identity_cycle_store(
    inner: Rc<dyn SeasonalIdentityCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalIdentityCycleStore> {
    Rc::new(CachedSeasonalIdentityCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalIdentityCycle>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal identity cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityCycleQuery {
    pub channel: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalIdentityCycleQuery {
    pub fn matches(&self, item: &SeasonalIdentityCycle) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
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

    pub fn apply(&self, items: &[SeasonalIdentityCycle]) -> Vec<SeasonalIdentityCycle> {
        let matched = collections::filter(items, |candidate: &SeasonalIdentityCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_identity_cycles(
    store: Rc<dyn SeasonalIdentityCycleStore>,
    query: &SeasonalIdentityCycleQuery,
) -> Vec<SeasonalIdentityCycle> {
    query.apply(&store.list())
}
