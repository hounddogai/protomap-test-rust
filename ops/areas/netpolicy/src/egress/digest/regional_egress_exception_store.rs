use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional egress exceptions by code.
pub trait RegionalEgressExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalEgressException>;
    fn save(&self, item: &RegionalEgressException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalEgressException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalEgressExceptionStore {
    pub items: RefCell<BTreeMap<String, RegionalEgressException>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalEgressExceptionStore for MemoryRegionalEgressExceptionStore {
    fn find(&self, code: &str) -> Option<RegionalEgressException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalEgressException) {
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

    fn list(&self) -> Vec<RegionalEgressException> {
        let mut out: Vec<RegionalEgressException> = Vec::new();
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

pub fn new_memory_regional_egress_exception_store() -> Rc<dyn RegionalEgressExceptionStore> {
    Rc::new(MemoryRegionalEgressExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalEgressExceptionStore {
    pub inner: Rc<dyn RegionalEgressExceptionStore>,
    pub cache: RefCell<cache::TtlCache<RegionalEgressException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalEgressExceptionStore for CachedRegionalEgressExceptionStore {
    fn find(&self, code: &str) -> Option<RegionalEgressException> {
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

    fn save(&self, item: &RegionalEgressException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalEgressException> {
        self.inner.list()
    }
}

pub fn new_cached_regional_egress_exception_store(
    inner: Rc<dyn RegionalEgressExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalEgressExceptionStore> {
    Rc::new(CachedRegionalEgressExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalEgressException>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional egress exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEgressExceptionQuery {
    pub name: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalEgressExceptionQuery {
    pub fn matches(&self, item: &RegionalEgressException) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalEgressException]) -> Vec<RegionalEgressException> {
        let matched = collections::filter(items, |candidate: &RegionalEgressException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_egress_exceptions(
    store: Rc<dyn RegionalEgressExceptionStore>,
    query: &RegionalEgressExceptionQuery,
) -> Vec<RegionalEgressException> {
    query.apply(&store.list())
}
