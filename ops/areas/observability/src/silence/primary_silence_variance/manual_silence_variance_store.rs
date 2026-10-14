use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual silence variances by code.
pub trait ManualSilenceVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualSilenceVariance>;
    fn save(&self, item: &ManualSilenceVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualSilenceVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualSilenceVarianceStore {
    pub items: RefCell<BTreeMap<String, ManualSilenceVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualSilenceVarianceStore for MemoryManualSilenceVarianceStore {
    fn find(&self, code: &str) -> Option<ManualSilenceVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualSilenceVariance) {
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

    fn list(&self) -> Vec<ManualSilenceVariance> {
        let mut out: Vec<ManualSilenceVariance> = Vec::new();
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

pub fn new_memory_manual_silence_variance_store() -> Rc<dyn ManualSilenceVarianceStore> {
    Rc::new(MemoryManualSilenceVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualSilenceVarianceStore {
    pub inner: Rc<dyn ManualSilenceVarianceStore>,
    pub cache: RefCell<cache::TtlCache<ManualSilenceVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualSilenceVarianceStore for CachedManualSilenceVarianceStore {
    fn find(&self, code: &str) -> Option<ManualSilenceVariance> {
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

    fn save(&self, item: &ManualSilenceVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualSilenceVariance> {
        self.inner.list()
    }
}

pub fn new_cached_manual_silence_variance_store(
    inner: Rc<dyn ManualSilenceVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualSilenceVarianceStore> {
    Rc::new(CachedManualSilenceVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualSilenceVariance>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual silence variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceVarianceQuery {
    pub label: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualSilenceVarianceQuery {
    pub fn matches(&self, item: &ManualSilenceVariance) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualSilenceVariance]) -> Vec<ManualSilenceVariance> {
        let matched = collections::filter(items, |candidate: &ManualSilenceVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_silence_variances(
    store: Rc<dyn ManualSilenceVarianceStore>,
    query: &ManualSilenceVarianceQuery,
) -> Vec<ManualSilenceVariance> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualSilenceVarianceStore> = new_memory_manual_silence_variance_store();
        let item = new_manual_silence_variance("code-966", "label-907");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
