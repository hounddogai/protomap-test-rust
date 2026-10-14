use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual credential quotas by code.
pub trait ManualCredentialQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualCredentialQuota>;
    fn save(&self, item: &ManualCredentialQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualCredentialQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualCredentialQuotaStore {
    pub items: RefCell<BTreeMap<String, ManualCredentialQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualCredentialQuotaStore for MemoryManualCredentialQuotaStore {
    fn find(&self, code: &str) -> Option<ManualCredentialQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualCredentialQuota) {
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

    fn list(&self) -> Vec<ManualCredentialQuota> {
        let mut out: Vec<ManualCredentialQuota> = Vec::new();
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

pub fn new_memory_manual_credential_quota_store() -> Rc<dyn ManualCredentialQuotaStore> {
    Rc::new(MemoryManualCredentialQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualCredentialQuotaStore {
    pub inner: Rc<dyn ManualCredentialQuotaStore>,
    pub cache: RefCell<cache::TtlCache<ManualCredentialQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualCredentialQuotaStore for CachedManualCredentialQuotaStore {
    fn find(&self, code: &str) -> Option<ManualCredentialQuota> {
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

    fn save(&self, item: &ManualCredentialQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualCredentialQuota> {
        self.inner.list()
    }
}

pub fn new_cached_manual_credential_quota_store(
    inner: Rc<dyn ManualCredentialQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualCredentialQuotaStore> {
    Rc::new(CachedManualCredentialQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualCredentialQuota>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual credential quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialQuotaQuery {
    pub channel: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualCredentialQuotaQuery {
    pub fn matches(&self, item: &ManualCredentialQuota) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualCredentialQuota]) -> Vec<ManualCredentialQuota> {
        let matched = collections::filter(items, |candidate: &ManualCredentialQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_credential_quotas(
    store: Rc<dyn ManualCredentialQuotaStore>,
    query: &ManualCredentialQuotaQuery,
) -> Vec<ManualCredentialQuota> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualCredentialQuotaStore> = new_memory_manual_credential_quota_store();
        let item = new_manual_credential_quota("code-986", "name-177");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
