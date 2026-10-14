use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered postmortem cycles by code.
pub trait TieredPostmortemCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredPostmortemCycle>;
    fn save(&self, item: &TieredPostmortemCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredPostmortemCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredPostmortemCycleStore {
    pub items: RefCell<BTreeMap<String, TieredPostmortemCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredPostmortemCycleStore for MemoryTieredPostmortemCycleStore {
    fn find(&self, code: &str) -> Option<TieredPostmortemCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredPostmortemCycle) {
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

    fn list(&self) -> Vec<TieredPostmortemCycle> {
        let mut out: Vec<TieredPostmortemCycle> = Vec::new();
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

pub fn new_memory_tiered_postmortem_cycle_store() -> Rc<dyn TieredPostmortemCycleStore> {
    Rc::new(MemoryTieredPostmortemCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredPostmortemCycleStore {
    pub inner: Rc<dyn TieredPostmortemCycleStore>,
    pub cache: RefCell<cache::TtlCache<TieredPostmortemCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredPostmortemCycleStore for CachedTieredPostmortemCycleStore {
    fn find(&self, code: &str) -> Option<TieredPostmortemCycle> {
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

    fn save(&self, item: &TieredPostmortemCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredPostmortemCycle> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_postmortem_cycle_store(
    inner: Rc<dyn TieredPostmortemCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredPostmortemCycleStore> {
    Rc::new(CachedTieredPostmortemCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredPostmortemCycle>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered postmortem cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemCycleQuery {
    pub segment: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredPostmortemCycleQuery {
    pub fn matches(&self, item: &TieredPostmortemCycle) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredPostmortemCycle]) -> Vec<TieredPostmortemCycle> {
        let matched = collections::filter(items, |candidate: &TieredPostmortemCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_postmortem_cycles(
    store: Rc<dyn TieredPostmortemCycleStore>,
    query: &TieredPostmortemCycleQuery,
) -> Vec<TieredPostmortemCycle> {
    query.apply(&store.list())
}
