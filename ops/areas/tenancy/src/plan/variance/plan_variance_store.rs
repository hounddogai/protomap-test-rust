use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps plan variances by code.
pub trait PlanVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PlanVariance>;
    fn save(&self, item: &PlanVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PlanVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPlanVarianceStore {
    pub items: RefCell<BTreeMap<String, PlanVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl PlanVarianceStore for MemoryPlanVarianceStore {
    fn find(&self, code: &str) -> Option<PlanVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PlanVariance) {
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

    fn list(&self) -> Vec<PlanVariance> {
        let mut out: Vec<PlanVariance> = Vec::new();
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

pub fn new_memory_plan_variance_store() -> Rc<dyn PlanVarianceStore> {
    Rc::new(MemoryPlanVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPlanVarianceStore {
    pub inner: Rc<dyn PlanVarianceStore>,
    pub cache: RefCell<cache::TtlCache<PlanVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PlanVarianceStore for CachedPlanVarianceStore {
    fn find(&self, code: &str) -> Option<PlanVariance> {
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

    fn save(&self, item: &PlanVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PlanVariance> {
        self.inner.list()
    }
}

pub fn new_cached_plan_variance_store(
    inner: Rc<dyn PlanVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PlanVarianceStore> {
    Rc::new(CachedPlanVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PlanVariance>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects plan variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanVarianceQuery {
    pub channel: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PlanVarianceQuery {
    pub fn matches(&self, item: &PlanVariance) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
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

    pub fn apply(&self, items: &[PlanVariance]) -> Vec<PlanVariance> {
        let matched = collections::filter(items, |candidate: &PlanVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_plan_variances(store: Rc<dyn PlanVarianceStore>, query: &PlanVarianceQuery) -> Vec<PlanVariance> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PlanVarianceStore> = new_memory_plan_variance_store();
        let item = new_plan_variance("code-833", "title-173");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
