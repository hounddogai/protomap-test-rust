use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default rollout variances by code.
pub trait DefaultRolloutVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultRolloutVariance>;
    fn save(&self, item: &DefaultRolloutVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultRolloutVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultRolloutVarianceStore {
    pub items: RefCell<BTreeMap<String, DefaultRolloutVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultRolloutVarianceStore for MemoryDefaultRolloutVarianceStore {
    fn find(&self, code: &str) -> Option<DefaultRolloutVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultRolloutVariance) {
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

    fn list(&self) -> Vec<DefaultRolloutVariance> {
        let mut out: Vec<DefaultRolloutVariance> = Vec::new();
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

pub fn new_memory_default_rollout_variance_store() -> Rc<dyn DefaultRolloutVarianceStore> {
    Rc::new(MemoryDefaultRolloutVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultRolloutVarianceStore {
    pub inner: Rc<dyn DefaultRolloutVarianceStore>,
    pub cache: RefCell<cache::TtlCache<DefaultRolloutVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultRolloutVarianceStore for CachedDefaultRolloutVarianceStore {
    fn find(&self, code: &str) -> Option<DefaultRolloutVariance> {
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

    fn save(&self, item: &DefaultRolloutVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultRolloutVariance> {
        self.inner.list()
    }
}

pub fn new_cached_default_rollout_variance_store(
    inner: Rc<dyn DefaultRolloutVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultRolloutVarianceStore> {
    Rc::new(CachedDefaultRolloutVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultRolloutVariance>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default rollout variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutVarianceQuery {
    pub category: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultRolloutVarianceQuery {
    pub fn matches(&self, item: &DefaultRolloutVariance) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultRolloutVariance]) -> Vec<DefaultRolloutVariance> {
        let matched = collections::filter(items, |candidate: &DefaultRolloutVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_rollout_variances(
    store: Rc<dyn DefaultRolloutVarianceStore>,
    query: &DefaultRolloutVarianceQuery,
) -> Vec<DefaultRolloutVariance> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DefaultRolloutVarianceStore> = new_memory_default_rollout_variance_store();
        let item = new_default_rollout_variance("code-445", "title-501");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
