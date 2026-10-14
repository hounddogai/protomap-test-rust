use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps alert adjustments by code.
pub trait AlertAdjustmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AlertAdjustment>;
    fn save(&self, item: &AlertAdjustment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AlertAdjustment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAlertAdjustmentStore {
    pub items: RefCell<BTreeMap<String, AlertAdjustment>>,
    pub order: RefCell<Vec<String>>,
}

impl AlertAdjustmentStore for MemoryAlertAdjustmentStore {
    fn find(&self, code: &str) -> Option<AlertAdjustment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AlertAdjustment) {
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

    fn list(&self) -> Vec<AlertAdjustment> {
        let mut out: Vec<AlertAdjustment> = Vec::new();
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

pub fn new_memory_alert_adjustment_store() -> Rc<dyn AlertAdjustmentStore> {
    Rc::new(MemoryAlertAdjustmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAlertAdjustmentStore {
    pub inner: Rc<dyn AlertAdjustmentStore>,
    pub cache: RefCell<cache::TtlCache<AlertAdjustment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AlertAdjustmentStore for CachedAlertAdjustmentStore {
    fn find(&self, code: &str) -> Option<AlertAdjustment> {
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

    fn save(&self, item: &AlertAdjustment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AlertAdjustment> {
        self.inner.list()
    }
}

pub fn new_cached_alert_adjustment_store(
    inner: Rc<dyn AlertAdjustmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AlertAdjustmentStore> {
    Rc::new(CachedAlertAdjustmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AlertAdjustment>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects alert adjustments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertAdjustmentQuery {
    pub segment: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AlertAdjustmentQuery {
    pub fn matches(&self, item: &AlertAdjustment) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AlertAdjustment]) -> Vec<AlertAdjustment> {
        let matched = collections::filter(items, |candidate: &AlertAdjustment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_alert_adjustments(
    store: Rc<dyn AlertAdjustmentStore>,
    query: &AlertAdjustmentQuery,
) -> Vec<AlertAdjustment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AlertAdjustmentStore> = new_memory_alert_adjustment_store();
        let item = new_alert_adjustment("code-29", "label-563");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
