use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps retention quotas by code.
pub trait RetentionQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RetentionQuota>;
    fn save(&self, item: &RetentionQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RetentionQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRetentionQuotaStore {
    pub items: RefCell<BTreeMap<String, RetentionQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl RetentionQuotaStore for MemoryRetentionQuotaStore {
    fn find(&self, code: &str) -> Option<RetentionQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RetentionQuota) {
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

    fn list(&self) -> Vec<RetentionQuota> {
        let mut out: Vec<RetentionQuota> = Vec::new();
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

pub fn new_memory_retention_quota_store() -> Rc<dyn RetentionQuotaStore> {
    Rc::new(MemoryRetentionQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRetentionQuotaStore {
    pub inner: Rc<dyn RetentionQuotaStore>,
    pub cache: RefCell<cache::TtlCache<RetentionQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RetentionQuotaStore for CachedRetentionQuotaStore {
    fn find(&self, code: &str) -> Option<RetentionQuota> {
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

    fn save(&self, item: &RetentionQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RetentionQuota> {
        self.inner.list()
    }
}

pub fn new_cached_retention_quota_store(
    inner: Rc<dyn RetentionQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RetentionQuotaStore> {
    Rc::new(CachedRetentionQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RetentionQuota>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects retention quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionQuotaQuery {
    pub scope: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RetentionQuotaQuery {
    pub fn matches(&self, item: &RetentionQuota) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RetentionQuota]) -> Vec<RetentionQuota> {
        let matched = collections::filter(items, |candidate: &RetentionQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_retention_quotas(store: Rc<dyn RetentionQuotaStore>, query: &RetentionQuotaQuery) -> Vec<RetentionQuota> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RetentionQuotaStore> = new_memory_retention_quota_store();
        let item = new_retention_quota("code-57", "name-551");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
