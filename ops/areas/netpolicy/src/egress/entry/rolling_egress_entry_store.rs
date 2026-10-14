use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling egress entries by code.
pub trait RollingEgressEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingEgressEntry>;
    fn save(&self, item: &RollingEgressEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingEgressEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingEgressEntryStore {
    pub items: RefCell<BTreeMap<String, RollingEgressEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingEgressEntryStore for MemoryRollingEgressEntryStore {
    fn find(&self, code: &str) -> Option<RollingEgressEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingEgressEntry) {
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

    fn list(&self) -> Vec<RollingEgressEntry> {
        let mut out: Vec<RollingEgressEntry> = Vec::new();
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

pub fn new_memory_rolling_egress_entry_store() -> Rc<dyn RollingEgressEntryStore> {
    Rc::new(MemoryRollingEgressEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingEgressEntryStore {
    pub inner: Rc<dyn RollingEgressEntryStore>,
    pub cache: RefCell<cache::TtlCache<RollingEgressEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingEgressEntryStore for CachedRollingEgressEntryStore {
    fn find(&self, code: &str) -> Option<RollingEgressEntry> {
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

    fn save(&self, item: &RollingEgressEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingEgressEntry> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_egress_entry_store(
    inner: Rc<dyn RollingEgressEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingEgressEntryStore> {
    Rc::new(CachedRollingEgressEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingEgressEntry>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling egress entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressEntryQuery {
    pub source: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingEgressEntryQuery {
    pub fn matches(&self, item: &RollingEgressEntry) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingEgressEntry]) -> Vec<RollingEgressEntry> {
        let matched = collections::filter(items, |candidate: &RollingEgressEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_egress_entries(
    store: Rc<dyn RollingEgressEntryStore>,
    query: &RollingEgressEntryQuery,
) -> Vec<RollingEgressEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingEgressEntryStore> = new_memory_rolling_egress_entry_store();
        let item = new_rolling_egress_entry("code-129", "label-972");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
