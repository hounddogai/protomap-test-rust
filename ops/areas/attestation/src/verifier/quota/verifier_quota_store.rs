use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps verifier quotas by code.
pub trait VerifierQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<VerifierQuota>;
    fn save(&self, item: &VerifierQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<VerifierQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryVerifierQuotaStore {
    pub items: RefCell<BTreeMap<String, VerifierQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl VerifierQuotaStore for MemoryVerifierQuotaStore {
    fn find(&self, code: &str) -> Option<VerifierQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &VerifierQuota) {
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

    fn list(&self) -> Vec<VerifierQuota> {
        let mut out: Vec<VerifierQuota> = Vec::new();
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

pub fn new_memory_verifier_quota_store() -> Rc<dyn VerifierQuotaStore> {
    Rc::new(MemoryVerifierQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedVerifierQuotaStore {
    pub inner: Rc<dyn VerifierQuotaStore>,
    pub cache: RefCell<cache::TtlCache<VerifierQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl VerifierQuotaStore for CachedVerifierQuotaStore {
    fn find(&self, code: &str) -> Option<VerifierQuota> {
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

    fn save(&self, item: &VerifierQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<VerifierQuota> {
        self.inner.list()
    }
}

pub fn new_cached_verifier_quota_store(
    inner: Rc<dyn VerifierQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn VerifierQuotaStore> {
    Rc::new(CachedVerifierQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<VerifierQuota>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects verifier quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierQuotaQuery {
    pub country: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl VerifierQuotaQuery {
    pub fn matches(&self, item: &VerifierQuota) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[VerifierQuota]) -> Vec<VerifierQuota> {
        let matched = collections::filter(items, |candidate: &VerifierQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_verifier_quotas(store: Rc<dyn VerifierQuotaStore>, query: &VerifierQuotaQuery) -> Vec<VerifierQuota> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn VerifierQuotaStore> = new_memory_verifier_quota_store();
        let item = new_verifier_quota("code-802", "label-920");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
