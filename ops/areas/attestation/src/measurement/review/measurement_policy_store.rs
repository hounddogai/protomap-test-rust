use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps measurement policies by code.
pub trait MeasurementPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<MeasurementPolicy>;
    fn save(&self, item: &MeasurementPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<MeasurementPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryMeasurementPolicyStore {
    pub items: RefCell<BTreeMap<String, MeasurementPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl MeasurementPolicyStore for MemoryMeasurementPolicyStore {
    fn find(&self, code: &str) -> Option<MeasurementPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &MeasurementPolicy) {
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

    fn list(&self) -> Vec<MeasurementPolicy> {
        let mut out: Vec<MeasurementPolicy> = Vec::new();
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

pub fn new_memory_measurement_policy_store() -> Rc<dyn MeasurementPolicyStore> {
    Rc::new(MemoryMeasurementPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedMeasurementPolicyStore {
    pub inner: Rc<dyn MeasurementPolicyStore>,
    pub cache: RefCell<cache::TtlCache<MeasurementPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl MeasurementPolicyStore for CachedMeasurementPolicyStore {
    fn find(&self, code: &str) -> Option<MeasurementPolicy> {
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

    fn save(&self, item: &MeasurementPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<MeasurementPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_measurement_policy_store(
    inner: Rc<dyn MeasurementPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn MeasurementPolicyStore> {
    Rc::new(CachedMeasurementPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<MeasurementPolicy>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects measurement policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPolicyQuery {
    pub title: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl MeasurementPolicyQuery {
    pub fn matches(&self, item: &MeasurementPolicy) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[MeasurementPolicy]) -> Vec<MeasurementPolicy> {
        let matched = collections::filter(items, |candidate: &MeasurementPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_measurement_policies(
    store: Rc<dyn MeasurementPolicyStore>,
    query: &MeasurementPolicyQuery,
) -> Vec<MeasurementPolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn MeasurementPolicyStore> = new_memory_measurement_policy_store();
        let item = new_measurement_policy("code-470", "title-569");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
