use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy egress rollups by code.
pub trait LegacyEgressRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyEgressRollup>;
    fn save(&self, item: &LegacyEgressRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyEgressRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyEgressRollupStore {
    pub items: RefCell<BTreeMap<String, LegacyEgressRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyEgressRollupStore for MemoryLegacyEgressRollupStore {
    fn find(&self, code: &str) -> Option<LegacyEgressRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyEgressRollup) {
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

    fn list(&self) -> Vec<LegacyEgressRollup> {
        let mut out: Vec<LegacyEgressRollup> = Vec::new();
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

pub fn new_memory_legacy_egress_rollup_store() -> Rc<dyn LegacyEgressRollupStore> {
    Rc::new(MemoryLegacyEgressRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyEgressRollupStore {
    pub inner: Rc<dyn LegacyEgressRollupStore>,
    pub cache: RefCell<cache::TtlCache<LegacyEgressRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyEgressRollupStore for CachedLegacyEgressRollupStore {
    fn find(&self, code: &str) -> Option<LegacyEgressRollup> {
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

    fn save(&self, item: &LegacyEgressRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyEgressRollup> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_egress_rollup_store(
    inner: Rc<dyn LegacyEgressRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyEgressRollupStore> {
    Rc::new(CachedLegacyEgressRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyEgressRollup>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy egress rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEgressRollupQuery {
    pub region: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyEgressRollupQuery {
    pub fn matches(&self, item: &LegacyEgressRollup) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyEgressRollup]) -> Vec<LegacyEgressRollup> {
        let matched = collections::filter(items, |candidate: &LegacyEgressRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_egress_rollups(
    store: Rc<dyn LegacyEgressRollupStore>,
    query: &LegacyEgressRollupQuery,
) -> Vec<LegacyEgressRollup> {
    query.apply(&store.list())
}
