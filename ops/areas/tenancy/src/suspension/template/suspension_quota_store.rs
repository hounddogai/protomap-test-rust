use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps suspension quotas by code.
pub trait SuspensionQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SuspensionQuota>;
    fn save(&self, item: &SuspensionQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SuspensionQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySuspensionQuotaStore {
    pub items: RefCell<BTreeMap<String, SuspensionQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl SuspensionQuotaStore for MemorySuspensionQuotaStore {
    fn find(&self, code: &str) -> Option<SuspensionQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SuspensionQuota) {
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

    fn list(&self) -> Vec<SuspensionQuota> {
        let mut out: Vec<SuspensionQuota> = Vec::new();
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

pub fn new_memory_suspension_quota_store() -> Rc<dyn SuspensionQuotaStore> {
    Rc::new(MemorySuspensionQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSuspensionQuotaStore {
    pub inner: Rc<dyn SuspensionQuotaStore>,
    pub cache: RefCell<cache::TtlCache<SuspensionQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SuspensionQuotaStore for CachedSuspensionQuotaStore {
    fn find(&self, code: &str) -> Option<SuspensionQuota> {
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

    fn save(&self, item: &SuspensionQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SuspensionQuota> {
        self.inner.list()
    }
}

pub fn new_cached_suspension_quota_store(
    inner: Rc<dyn SuspensionQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SuspensionQuotaStore> {
    Rc::new(CachedSuspensionQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SuspensionQuota>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects suspension quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionQuotaQuery {
    pub source: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SuspensionQuotaQuery {
    pub fn matches(&self, item: &SuspensionQuota) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SuspensionQuota]) -> Vec<SuspensionQuota> {
        let matched = collections::filter(items, |candidate: &SuspensionQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_suspension_quotas(
    store: Rc<dyn SuspensionQuotaStore>,
    query: &SuspensionQuotaQuery,
) -> Vec<SuspensionQuota> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SuspensionQuotaStore> = new_memory_suspension_quota_store();
        let item = new_suspension_quota("code-18", "label-448");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
