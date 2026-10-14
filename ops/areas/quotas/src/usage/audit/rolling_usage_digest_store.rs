use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling usage digests by code.
pub trait RollingUsageDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingUsageDigest>;
    fn save(&self, item: &RollingUsageDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingUsageDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingUsageDigestStore {
    pub items: RefCell<BTreeMap<String, RollingUsageDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingUsageDigestStore for MemoryRollingUsageDigestStore {
    fn find(&self, code: &str) -> Option<RollingUsageDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingUsageDigest) {
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

    fn list(&self) -> Vec<RollingUsageDigest> {
        let mut out: Vec<RollingUsageDigest> = Vec::new();
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

pub fn new_memory_rolling_usage_digest_store() -> Rc<dyn RollingUsageDigestStore> {
    Rc::new(MemoryRollingUsageDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingUsageDigestStore {
    pub inner: Rc<dyn RollingUsageDigestStore>,
    pub cache: RefCell<cache::TtlCache<RollingUsageDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingUsageDigestStore for CachedRollingUsageDigestStore {
    fn find(&self, code: &str) -> Option<RollingUsageDigest> {
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

    fn save(&self, item: &RollingUsageDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingUsageDigest> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_usage_digest_store(
    inner: Rc<dyn RollingUsageDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingUsageDigestStore> {
    Rc::new(CachedRollingUsageDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingUsageDigest>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling usage digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingUsageDigestQuery {
    pub segment: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingUsageDigestQuery {
    pub fn matches(&self, item: &RollingUsageDigest) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.expires_at < self.min_expires_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingUsageDigest]) -> Vec<RollingUsageDigest> {
        let matched = collections::filter(items, |candidate: &RollingUsageDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_usage_digests(
    store: Rc<dyn RollingUsageDigestStore>,
    query: &RollingUsageDigestQuery,
) -> Vec<RollingUsageDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingUsageDigestStore> = new_memory_rolling_usage_digest_store();
        let item = new_rolling_usage_digest("code-197", "title-825");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
