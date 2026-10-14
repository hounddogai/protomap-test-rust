use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps identity variances by code.
pub trait IdentityVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<IdentityVariance>;
    fn save(&self, item: &IdentityVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<IdentityVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryIdentityVarianceStore {
    pub items: RefCell<BTreeMap<String, IdentityVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl IdentityVarianceStore for MemoryIdentityVarianceStore {
    fn find(&self, code: &str) -> Option<IdentityVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &IdentityVariance) {
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

    fn list(&self) -> Vec<IdentityVariance> {
        let mut out: Vec<IdentityVariance> = Vec::new();
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

pub fn new_memory_identity_variance_store() -> Rc<dyn IdentityVarianceStore> {
    Rc::new(MemoryIdentityVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedIdentityVarianceStore {
    pub inner: Rc<dyn IdentityVarianceStore>,
    pub cache: RefCell<cache::TtlCache<IdentityVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl IdentityVarianceStore for CachedIdentityVarianceStore {
    fn find(&self, code: &str) -> Option<IdentityVariance> {
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

    fn save(&self, item: &IdentityVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<IdentityVariance> {
        self.inner.list()
    }
}

pub fn new_cached_identity_variance_store(
    inner: Rc<dyn IdentityVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn IdentityVarianceStore> {
    Rc::new(CachedIdentityVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<IdentityVariance>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects identity variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityVarianceQuery {
    pub region: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl IdentityVarianceQuery {
    pub fn matches(&self, item: &IdentityVariance) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[IdentityVariance]) -> Vec<IdentityVariance> {
        let matched = collections::filter(items, |candidate: &IdentityVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_identity_variances(
    store: Rc<dyn IdentityVarianceStore>,
    query: &IdentityVarianceQuery,
) -> Vec<IdentityVariance> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn IdentityVarianceStore> = new_memory_identity_variance_store();
        let item = new_identity_variance("code-309", "label-901");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
