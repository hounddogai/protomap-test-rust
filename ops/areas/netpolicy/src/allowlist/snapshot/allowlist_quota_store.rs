use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps allowlist quotas by code.
pub trait AllowlistQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AllowlistQuota>;
    fn save(&self, item: &AllowlistQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AllowlistQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAllowlistQuotaStore {
    pub items: RefCell<BTreeMap<String, AllowlistQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl AllowlistQuotaStore for MemoryAllowlistQuotaStore {
    fn find(&self, code: &str) -> Option<AllowlistQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AllowlistQuota) {
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

    fn list(&self) -> Vec<AllowlistQuota> {
        let mut out: Vec<AllowlistQuota> = Vec::new();
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

pub fn new_memory_allowlist_quota_store() -> Rc<dyn AllowlistQuotaStore> {
    Rc::new(MemoryAllowlistQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAllowlistQuotaStore {
    pub inner: Rc<dyn AllowlistQuotaStore>,
    pub cache: RefCell<cache::TtlCache<AllowlistQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AllowlistQuotaStore for CachedAllowlistQuotaStore {
    fn find(&self, code: &str) -> Option<AllowlistQuota> {
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

    fn save(&self, item: &AllowlistQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AllowlistQuota> {
        self.inner.list()
    }
}

pub fn new_cached_allowlist_quota_store(
    inner: Rc<dyn AllowlistQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AllowlistQuotaStore> {
    Rc::new(CachedAllowlistQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AllowlistQuota>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects allowlist quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistQuotaQuery {
    pub region: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AllowlistQuotaQuery {
    pub fn matches(&self, item: &AllowlistQuota) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AllowlistQuota]) -> Vec<AllowlistQuota> {
        let matched = collections::filter(items, |candidate: &AllowlistQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_allowlist_quotas(store: Rc<dyn AllowlistQuotaStore>, query: &AllowlistQuotaQuery) -> Vec<AllowlistQuota> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AllowlistQuotaStore> = new_memory_allowlist_quota_store();
        let item = new_allowlist_quota("code-118", "name-11");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
