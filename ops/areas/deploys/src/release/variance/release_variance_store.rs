use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps release variances by code.
pub trait ReleaseVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ReleaseVariance>;
    fn save(&self, item: &ReleaseVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ReleaseVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryReleaseVarianceStore {
    pub items: RefCell<BTreeMap<String, ReleaseVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl ReleaseVarianceStore for MemoryReleaseVarianceStore {
    fn find(&self, code: &str) -> Option<ReleaseVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ReleaseVariance) {
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

    fn list(&self) -> Vec<ReleaseVariance> {
        let mut out: Vec<ReleaseVariance> = Vec::new();
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

pub fn new_memory_release_variance_store() -> Rc<dyn ReleaseVarianceStore> {
    Rc::new(MemoryReleaseVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedReleaseVarianceStore {
    pub inner: Rc<dyn ReleaseVarianceStore>,
    pub cache: RefCell<cache::TtlCache<ReleaseVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ReleaseVarianceStore for CachedReleaseVarianceStore {
    fn find(&self, code: &str) -> Option<ReleaseVariance> {
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

    fn save(&self, item: &ReleaseVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ReleaseVariance> {
        self.inner.list()
    }
}

pub fn new_cached_release_variance_store(
    inner: Rc<dyn ReleaseVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ReleaseVarianceStore> {
    Rc::new(CachedReleaseVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ReleaseVariance>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects release variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseVarianceQuery {
    pub cluster: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ReleaseVarianceQuery {
    pub fn matches(&self, item: &ReleaseVariance) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[ReleaseVariance]) -> Vec<ReleaseVariance> {
        let matched = collections::filter(items, |candidate: &ReleaseVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_release_variances(
    store: Rc<dyn ReleaseVarianceStore>,
    query: &ReleaseVarianceQuery,
) -> Vec<ReleaseVariance> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ReleaseVarianceStore> = new_memory_release_variance_store();
        let item = new_release_variance("code-908", "label-155");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
