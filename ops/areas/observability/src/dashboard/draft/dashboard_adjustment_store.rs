use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dashboard adjustments by code.
pub trait DashboardAdjustmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DashboardAdjustment>;
    fn save(&self, item: &DashboardAdjustment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DashboardAdjustment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDashboardAdjustmentStore {
    pub items: RefCell<BTreeMap<String, DashboardAdjustment>>,
    pub order: RefCell<Vec<String>>,
}

impl DashboardAdjustmentStore for MemoryDashboardAdjustmentStore {
    fn find(&self, code: &str) -> Option<DashboardAdjustment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DashboardAdjustment) {
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

    fn list(&self) -> Vec<DashboardAdjustment> {
        let mut out: Vec<DashboardAdjustment> = Vec::new();
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

pub fn new_memory_dashboard_adjustment_store() -> Rc<dyn DashboardAdjustmentStore> {
    Rc::new(MemoryDashboardAdjustmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDashboardAdjustmentStore {
    pub inner: Rc<dyn DashboardAdjustmentStore>,
    pub cache: RefCell<cache::TtlCache<DashboardAdjustment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DashboardAdjustmentStore for CachedDashboardAdjustmentStore {
    fn find(&self, code: &str) -> Option<DashboardAdjustment> {
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

    fn save(&self, item: &DashboardAdjustment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DashboardAdjustment> {
        self.inner.list()
    }
}

pub fn new_cached_dashboard_adjustment_store(
    inner: Rc<dyn DashboardAdjustmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DashboardAdjustmentStore> {
    Rc::new(CachedDashboardAdjustmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DashboardAdjustment>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dashboard adjustments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAdjustmentQuery {
    pub title: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DashboardAdjustmentQuery {
    pub fn matches(&self, item: &DashboardAdjustment) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[DashboardAdjustment]) -> Vec<DashboardAdjustment> {
        let matched = collections::filter(items, |candidate: &DashboardAdjustment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dashboard_adjustments(
    store: Rc<dyn DashboardAdjustmentStore>,
    query: &DashboardAdjustmentQuery,
) -> Vec<DashboardAdjustment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DashboardAdjustmentStore> = new_memory_dashboard_adjustment_store();
        let item = new_dashboard_adjustment("code-758", "title-585");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
