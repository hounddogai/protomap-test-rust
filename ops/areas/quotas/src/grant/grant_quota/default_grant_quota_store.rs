use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default grant quotas by code.
pub trait DefaultGrantQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultGrantQuota>;
    fn save(&self, item: &DefaultGrantQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultGrantQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultGrantQuotaStore {
    pub items: RefCell<BTreeMap<String, DefaultGrantQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultGrantQuotaStore for MemoryDefaultGrantQuotaStore {
    fn find(&self, code: &str) -> Option<DefaultGrantQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultGrantQuota) {
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

    fn list(&self) -> Vec<DefaultGrantQuota> {
        let mut out: Vec<DefaultGrantQuota> = Vec::new();
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

pub fn new_memory_default_grant_quota_store() -> Rc<dyn DefaultGrantQuotaStore> {
    Rc::new(MemoryDefaultGrantQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultGrantQuotaStore {
    pub inner: Rc<dyn DefaultGrantQuotaStore>,
    pub cache: RefCell<cache::TtlCache<DefaultGrantQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultGrantQuotaStore for CachedDefaultGrantQuotaStore {
    fn find(&self, code: &str) -> Option<DefaultGrantQuota> {
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

    fn save(&self, item: &DefaultGrantQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultGrantQuota> {
        self.inner.list()
    }
}

pub fn new_cached_default_grant_quota_store(
    inner: Rc<dyn DefaultGrantQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultGrantQuotaStore> {
    Rc::new(CachedDefaultGrantQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultGrantQuota>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default grant quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultGrantQuotaQuery {
    pub country: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultGrantQuotaQuery {
    pub fn matches(&self, item: &DefaultGrantQuota) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultGrantQuota]) -> Vec<DefaultGrantQuota> {
        let matched = collections::filter(items, |candidate: &DefaultGrantQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_grant_quotas(
    store: Rc<dyn DefaultGrantQuotaStore>,
    query: &DefaultGrantQuotaQuery,
) -> Vec<DefaultGrantQuota> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DefaultGrantQuotaStore> = new_memory_default_grant_quota_store();
        let item = new_default_grant_quota("code-608", "name-620");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
