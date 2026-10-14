use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global tenant digests by code.
pub trait GlobalTenantDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalTenantDigest>;
    fn save(&self, item: &GlobalTenantDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalTenantDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalTenantDigestStore {
    pub items: RefCell<BTreeMap<String, GlobalTenantDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalTenantDigestStore for MemoryGlobalTenantDigestStore {
    fn find(&self, code: &str) -> Option<GlobalTenantDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalTenantDigest) {
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

    fn list(&self) -> Vec<GlobalTenantDigest> {
        let mut out: Vec<GlobalTenantDigest> = Vec::new();
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

pub fn new_memory_global_tenant_digest_store() -> Rc<dyn GlobalTenantDigestStore> {
    Rc::new(MemoryGlobalTenantDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalTenantDigestStore {
    pub inner: Rc<dyn GlobalTenantDigestStore>,
    pub cache: RefCell<cache::TtlCache<GlobalTenantDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalTenantDigestStore for CachedGlobalTenantDigestStore {
    fn find(&self, code: &str) -> Option<GlobalTenantDigest> {
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

    fn save(&self, item: &GlobalTenantDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalTenantDigest> {
        self.inner.list()
    }
}

pub fn new_cached_global_tenant_digest_store(
    inner: Rc<dyn GlobalTenantDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalTenantDigestStore> {
    Rc::new(CachedGlobalTenantDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalTenantDigest>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global tenant digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTenantDigestQuery {
    pub channel: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalTenantDigestQuery {
    pub fn matches(&self, item: &GlobalTenantDigest) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalTenantDigest]) -> Vec<GlobalTenantDigest> {
        let matched = collections::filter(items, |candidate: &GlobalTenantDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_tenant_digests(
    store: Rc<dyn GlobalTenantDigestStore>,
    query: &GlobalTenantDigestQuery,
) -> Vec<GlobalTenantDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalTenantDigestStore> = new_memory_global_tenant_digest_store();
        let item = new_global_tenant_digest("code-940", "name-127");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
