use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps peering quotas by code.
pub trait PeeringQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PeeringQuota>;
    fn save(&self, item: &PeeringQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PeeringQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPeeringQuotaStore {
    pub items: RefCell<BTreeMap<String, PeeringQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl PeeringQuotaStore for MemoryPeeringQuotaStore {
    fn find(&self, code: &str) -> Option<PeeringQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PeeringQuota) {
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

    fn list(&self) -> Vec<PeeringQuota> {
        let mut out: Vec<PeeringQuota> = Vec::new();
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

pub fn new_memory_peering_quota_store() -> Rc<dyn PeeringQuotaStore> {
    Rc::new(MemoryPeeringQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPeeringQuotaStore {
    pub inner: Rc<dyn PeeringQuotaStore>,
    pub cache: RefCell<cache::TtlCache<PeeringQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PeeringQuotaStore for CachedPeeringQuotaStore {
    fn find(&self, code: &str) -> Option<PeeringQuota> {
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

    fn save(&self, item: &PeeringQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PeeringQuota> {
        self.inner.list()
    }
}

pub fn new_cached_peering_quota_store(
    inner: Rc<dyn PeeringQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PeeringQuotaStore> {
    Rc::new(CachedPeeringQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PeeringQuota>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects peering quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringQuotaQuery {
    pub label: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PeeringQuotaQuery {
    pub fn matches(&self, item: &PeeringQuota) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PeeringQuota]) -> Vec<PeeringQuota> {
        let matched = collections::filter(items, |candidate: &PeeringQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_peering_quotas(store: Rc<dyn PeeringQuotaStore>, query: &PeeringQuotaQuery) -> Vec<PeeringQuota> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PeeringQuotaStore> = new_memory_peering_quota_store();
        let item = new_peering_quota("code-466", "label-62");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
