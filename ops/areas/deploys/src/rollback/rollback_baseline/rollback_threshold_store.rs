use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rollback thresholds by code.
pub trait RollbackThresholdStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollbackThreshold>;
    fn save(&self, item: &RollbackThreshold);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollbackThreshold>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollbackThresholdStore {
    pub items: RefCell<BTreeMap<String, RollbackThreshold>>,
    pub order: RefCell<Vec<String>>,
}

impl RollbackThresholdStore for MemoryRollbackThresholdStore {
    fn find(&self, code: &str) -> Option<RollbackThreshold> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollbackThreshold) {
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

    fn list(&self) -> Vec<RollbackThreshold> {
        let mut out: Vec<RollbackThreshold> = Vec::new();
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

pub fn new_memory_rollback_threshold_store() -> Rc<dyn RollbackThresholdStore> {
    Rc::new(MemoryRollbackThresholdStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollbackThresholdStore {
    pub inner: Rc<dyn RollbackThresholdStore>,
    pub cache: RefCell<cache::TtlCache<RollbackThreshold>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollbackThresholdStore for CachedRollbackThresholdStore {
    fn find(&self, code: &str) -> Option<RollbackThreshold> {
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

    fn save(&self, item: &RollbackThreshold) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollbackThreshold> {
        self.inner.list()
    }
}

pub fn new_cached_rollback_threshold_store(
    inner: Rc<dyn RollbackThresholdStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollbackThresholdStore> {
    Rc::new(CachedRollbackThresholdStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollbackThreshold>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rollback thresholds. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackThresholdQuery {
    pub channel: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollbackThresholdQuery {
    pub fn matches(&self, item: &RollbackThreshold) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.ends_at < self.min_ends_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollbackThreshold]) -> Vec<RollbackThreshold> {
        let matched = collections::filter(items, |candidate: &RollbackThreshold| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rollback_thresholds(
    store: Rc<dyn RollbackThresholdStore>,
    query: &RollbackThresholdQuery,
) -> Vec<RollbackThreshold> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollbackThresholdStore> = new_memory_rollback_threshold_store();
        let item = new_rollback_threshold("code-87", "label-98");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
