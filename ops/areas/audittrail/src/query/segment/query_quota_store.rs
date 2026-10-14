use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps query quotas by code.
pub trait QueryQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QueryQuota>;
    fn save(&self, item: &QueryQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QueryQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQueryQuotaStore {
    pub items: RefCell<BTreeMap<String, QueryQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl QueryQuotaStore for MemoryQueryQuotaStore {
    fn find(&self, code: &str) -> Option<QueryQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QueryQuota) {
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

    fn list(&self) -> Vec<QueryQuota> {
        let mut out: Vec<QueryQuota> = Vec::new();
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

pub fn new_memory_query_quota_store() -> Rc<dyn QueryQuotaStore> {
    Rc::new(MemoryQueryQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQueryQuotaStore {
    pub inner: Rc<dyn QueryQuotaStore>,
    pub cache: RefCell<cache::TtlCache<QueryQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QueryQuotaStore for CachedQueryQuotaStore {
    fn find(&self, code: &str) -> Option<QueryQuota> {
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

    fn save(&self, item: &QueryQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QueryQuota> {
        self.inner.list()
    }
}

pub fn new_cached_query_quota_store(
    inner: Rc<dyn QueryQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QueryQuotaStore> {
    Rc::new(CachedQueryQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QueryQuota>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects query quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryQuotaQuery {
    pub label: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QueryQuotaQuery {
    pub fn matches(&self, item: &QueryQuota) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[QueryQuota]) -> Vec<QueryQuota> {
        let matched = collections::filter(items, |candidate: &QueryQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_query_quotas(store: Rc<dyn QueryQuotaStore>, query: &QueryQuotaQuery) -> Vec<QueryQuota> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn QueryQuotaStore> = new_memory_query_quota_store();
        let item = new_query_quota("code-162", "label-845");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
