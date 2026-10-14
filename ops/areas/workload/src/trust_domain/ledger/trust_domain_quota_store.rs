use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps trust domain quotas by code.
pub trait TrustDomainQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TrustDomainQuota>;
    fn save(&self, item: &TrustDomainQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TrustDomainQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTrustDomainQuotaStore {
    pub items: RefCell<BTreeMap<String, TrustDomainQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl TrustDomainQuotaStore for MemoryTrustDomainQuotaStore {
    fn find(&self, code: &str) -> Option<TrustDomainQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TrustDomainQuota) {
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

    fn list(&self) -> Vec<TrustDomainQuota> {
        let mut out: Vec<TrustDomainQuota> = Vec::new();
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

pub fn new_memory_trust_domain_quota_store() -> Rc<dyn TrustDomainQuotaStore> {
    Rc::new(MemoryTrustDomainQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTrustDomainQuotaStore {
    pub inner: Rc<dyn TrustDomainQuotaStore>,
    pub cache: RefCell<cache::TtlCache<TrustDomainQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TrustDomainQuotaStore for CachedTrustDomainQuotaStore {
    fn find(&self, code: &str) -> Option<TrustDomainQuota> {
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

    fn save(&self, item: &TrustDomainQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TrustDomainQuota> {
        self.inner.list()
    }
}

pub fn new_cached_trust_domain_quota_store(
    inner: Rc<dyn TrustDomainQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TrustDomainQuotaStore> {
    Rc::new(CachedTrustDomainQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TrustDomainQuota>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects trust domain quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainQuotaQuery {
    pub channel: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TrustDomainQuotaQuery {
    pub fn matches(&self, item: &TrustDomainQuota) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
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

    pub fn apply(&self, items: &[TrustDomainQuota]) -> Vec<TrustDomainQuota> {
        let matched = collections::filter(items, |candidate: &TrustDomainQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_trust_domain_quotas(
    store: Rc<dyn TrustDomainQuotaStore>,
    query: &TrustDomainQuotaQuery,
) -> Vec<TrustDomainQuota> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TrustDomainQuotaStore> = new_memory_trust_domain_quota_store();
        let item = new_trust_domain_quota("code-362", "name-849");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
