use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global sink quotas by code.
pub trait GlobalSinkQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalSinkQuota>;
    fn save(&self, item: &GlobalSinkQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalSinkQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalSinkQuotaStore {
    pub items: RefCell<BTreeMap<String, GlobalSinkQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalSinkQuotaStore for MemoryGlobalSinkQuotaStore {
    fn find(&self, code: &str) -> Option<GlobalSinkQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalSinkQuota) {
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

    fn list(&self) -> Vec<GlobalSinkQuota> {
        let mut out: Vec<GlobalSinkQuota> = Vec::new();
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

pub fn new_memory_global_sink_quota_store() -> Rc<dyn GlobalSinkQuotaStore> {
    Rc::new(MemoryGlobalSinkQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalSinkQuotaStore {
    pub inner: Rc<dyn GlobalSinkQuotaStore>,
    pub cache: RefCell<cache::TtlCache<GlobalSinkQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalSinkQuotaStore for CachedGlobalSinkQuotaStore {
    fn find(&self, code: &str) -> Option<GlobalSinkQuota> {
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

    fn save(&self, item: &GlobalSinkQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalSinkQuota> {
        self.inner.list()
    }
}

pub fn new_cached_global_sink_quota_store(
    inner: Rc<dyn GlobalSinkQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalSinkQuotaStore> {
    Rc::new(CachedGlobalSinkQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalSinkQuota>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global sink quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkQuotaQuery {
    pub label: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalSinkQuotaQuery {
    pub fn matches(&self, item: &GlobalSinkQuota) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[GlobalSinkQuota]) -> Vec<GlobalSinkQuota> {
        let matched = collections::filter(items, |candidate: &GlobalSinkQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_sink_quotas(
    store: Rc<dyn GlobalSinkQuotaStore>,
    query: &GlobalSinkQuotaQuery,
) -> Vec<GlobalSinkQuota> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalSinkQuotaStore> = new_memory_global_sink_quota_store();
        let item = new_global_sink_quota("code-414", "label-723");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
