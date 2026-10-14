use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps alert digests by code.
pub trait AlertDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AlertDigest>;
    fn save(&self, item: &AlertDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AlertDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAlertDigestStore {
    pub items: RefCell<BTreeMap<String, AlertDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl AlertDigestStore for MemoryAlertDigestStore {
    fn find(&self, code: &str) -> Option<AlertDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AlertDigest) {
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

    fn list(&self) -> Vec<AlertDigest> {
        let mut out: Vec<AlertDigest> = Vec::new();
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

pub fn new_memory_alert_digest_store() -> Rc<dyn AlertDigestStore> {
    Rc::new(MemoryAlertDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAlertDigestStore {
    pub inner: Rc<dyn AlertDigestStore>,
    pub cache: RefCell<cache::TtlCache<AlertDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AlertDigestStore for CachedAlertDigestStore {
    fn find(&self, code: &str) -> Option<AlertDigest> {
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

    fn save(&self, item: &AlertDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AlertDigest> {
        self.inner.list()
    }
}

pub fn new_cached_alert_digest_store(
    inner: Rc<dyn AlertDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AlertDigestStore> {
    Rc::new(CachedAlertDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AlertDigest>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects alert digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertDigestQuery {
    pub label: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AlertDigestQuery {
    pub fn matches(&self, item: &AlertDigest) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[AlertDigest]) -> Vec<AlertDigest> {
        let matched = collections::filter(items, |candidate: &AlertDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_alert_digests(store: Rc<dyn AlertDigestStore>, query: &AlertDigestQuery) -> Vec<AlertDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AlertDigestStore> = new_memory_alert_digest_store();
        let item = new_alert_digest("code-350", "label-712");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
