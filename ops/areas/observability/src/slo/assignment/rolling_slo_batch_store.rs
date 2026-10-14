use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling slo batches by code.
pub trait RollingSloBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingSloBatch>;
    fn save(&self, item: &RollingSloBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingSloBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingSloBatchStore {
    pub items: RefCell<BTreeMap<String, RollingSloBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingSloBatchStore for MemoryRollingSloBatchStore {
    fn find(&self, code: &str) -> Option<RollingSloBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingSloBatch) {
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

    fn list(&self) -> Vec<RollingSloBatch> {
        let mut out: Vec<RollingSloBatch> = Vec::new();
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

pub fn new_memory_rolling_slo_batch_store() -> Rc<dyn RollingSloBatchStore> {
    Rc::new(MemoryRollingSloBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingSloBatchStore {
    pub inner: Rc<dyn RollingSloBatchStore>,
    pub cache: RefCell<cache::TtlCache<RollingSloBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingSloBatchStore for CachedRollingSloBatchStore {
    fn find(&self, code: &str) -> Option<RollingSloBatch> {
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

    fn save(&self, item: &RollingSloBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingSloBatch> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_slo_batch_store(
    inner: Rc<dyn RollingSloBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingSloBatchStore> {
    Rc::new(CachedRollingSloBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingSloBatch>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling slo batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSloBatchQuery {
    pub name: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingSloBatchQuery {
    pub fn matches(&self, item: &RollingSloBatch) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingSloBatch]) -> Vec<RollingSloBatch> {
        let matched = collections::filter(items, |candidate: &RollingSloBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_slo_batches(
    store: Rc<dyn RollingSloBatchStore>,
    query: &RollingSloBatchQuery,
) -> Vec<RollingSloBatch> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingSloBatchStore> = new_memory_rolling_slo_batch_store();
        let item = new_rolling_slo_batch("code-319", "name-591");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
