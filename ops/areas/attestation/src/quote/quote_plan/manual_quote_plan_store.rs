use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual quote plans by code.
pub trait ManualQuotePlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualQuotePlan>;
    fn save(&self, item: &ManualQuotePlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualQuotePlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualQuotePlanStore {
    pub items: RefCell<BTreeMap<String, ManualQuotePlan>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualQuotePlanStore for MemoryManualQuotePlanStore {
    fn find(&self, code: &str) -> Option<ManualQuotePlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualQuotePlan) {
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

    fn list(&self) -> Vec<ManualQuotePlan> {
        let mut out: Vec<ManualQuotePlan> = Vec::new();
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

pub fn new_memory_manual_quote_plan_store() -> Rc<dyn ManualQuotePlanStore> {
    Rc::new(MemoryManualQuotePlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualQuotePlanStore {
    pub inner: Rc<dyn ManualQuotePlanStore>,
    pub cache: RefCell<cache::TtlCache<ManualQuotePlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualQuotePlanStore for CachedManualQuotePlanStore {
    fn find(&self, code: &str) -> Option<ManualQuotePlan> {
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

    fn save(&self, item: &ManualQuotePlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualQuotePlan> {
        self.inner.list()
    }
}

pub fn new_cached_manual_quote_plan_store(
    inner: Rc<dyn ManualQuotePlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualQuotePlanStore> {
    Rc::new(CachedManualQuotePlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualQuotePlan>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual quote plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuotePlanQuery {
    pub unit: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualQuotePlanQuery {
    pub fn matches(&self, item: &ManualQuotePlan) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
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

    pub fn apply(&self, items: &[ManualQuotePlan]) -> Vec<ManualQuotePlan> {
        let matched = collections::filter(items, |candidate: &ManualQuotePlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_quote_plans(
    store: Rc<dyn ManualQuotePlanStore>,
    query: &ManualQuotePlanQuery,
) -> Vec<ManualQuotePlan> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualQuotePlanStore> = new_memory_manual_quote_plan_store();
        let item = new_manual_quote_plan("code-745", "title-460");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
