use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending query quotas by code.
pub trait PendingQueryQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingQueryQuota>;
    fn save(&self, item: &PendingQueryQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingQueryQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingQueryQuotaStore {
    pub items: RefCell<BTreeMap<String, PendingQueryQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingQueryQuotaStore for MemoryPendingQueryQuotaStore {
    fn find(&self, code: &str) -> Option<PendingQueryQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingQueryQuota) {
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

    fn list(&self) -> Vec<PendingQueryQuota> {
        let mut out: Vec<PendingQueryQuota> = Vec::new();
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

pub fn new_memory_pending_query_quota_store() -> Rc<dyn PendingQueryQuotaStore> {
    Rc::new(MemoryPendingQueryQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingQueryQuotaStore {
    pub inner: Rc<dyn PendingQueryQuotaStore>,
    pub cache: RefCell<cache::TtlCache<PendingQueryQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingQueryQuotaStore for CachedPendingQueryQuotaStore {
    fn find(&self, code: &str) -> Option<PendingQueryQuota> {
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

    fn save(&self, item: &PendingQueryQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingQueryQuota> {
        self.inner.list()
    }
}

pub fn new_cached_pending_query_quota_store(
    inner: Rc<dyn PendingQueryQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingQueryQuotaStore> {
    Rc::new(CachedPendingQueryQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingQueryQuota>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending query quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryQuotaQuery {
    pub title: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingQueryQuotaQuery {
    pub fn matches(&self, item: &PendingQueryQuota) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[PendingQueryQuota]) -> Vec<PendingQueryQuota> {
        let matched = collections::filter(items, |candidate: &PendingQueryQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_query_quotas(
    store: Rc<dyn PendingQueryQuotaStore>,
    query: &PendingQueryQuotaQuery,
) -> Vec<PendingQueryQuota> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PendingQueryQuotaStore> = new_memory_pending_query_quota_store();
        let item = new_pending_query_quota("code-923", "title-94");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
