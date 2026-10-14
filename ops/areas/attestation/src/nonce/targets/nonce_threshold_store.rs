use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps nonce thresholds by code.
pub trait NonceThresholdStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<NonceThreshold>;
    fn save(&self, item: &NonceThreshold);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<NonceThreshold>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryNonceThresholdStore {
    pub items: RefCell<BTreeMap<String, NonceThreshold>>,
    pub order: RefCell<Vec<String>>,
}

impl NonceThresholdStore for MemoryNonceThresholdStore {
    fn find(&self, code: &str) -> Option<NonceThreshold> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &NonceThreshold) {
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

    fn list(&self) -> Vec<NonceThreshold> {
        let mut out: Vec<NonceThreshold> = Vec::new();
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

pub fn new_memory_nonce_threshold_store() -> Rc<dyn NonceThresholdStore> {
    Rc::new(MemoryNonceThresholdStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedNonceThresholdStore {
    pub inner: Rc<dyn NonceThresholdStore>,
    pub cache: RefCell<cache::TtlCache<NonceThreshold>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl NonceThresholdStore for CachedNonceThresholdStore {
    fn find(&self, code: &str) -> Option<NonceThreshold> {
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

    fn save(&self, item: &NonceThreshold) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<NonceThreshold> {
        self.inner.list()
    }
}

pub fn new_cached_nonce_threshold_store(
    inner: Rc<dyn NonceThresholdStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn NonceThresholdStore> {
    Rc::new(CachedNonceThresholdStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<NonceThreshold>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects nonce thresholds. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceThresholdQuery {
    pub name: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl NonceThresholdQuery {
    pub fn matches(&self, item: &NonceThreshold) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[NonceThreshold]) -> Vec<NonceThreshold> {
        let matched = collections::filter(items, |candidate: &NonceThreshold| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_nonce_thresholds(store: Rc<dyn NonceThresholdStore>, query: &NonceThresholdQuery) -> Vec<NonceThreshold> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn NonceThresholdStore> = new_memory_nonce_threshold_store();
        let item = new_nonce_threshold("code-616", "name-172");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
