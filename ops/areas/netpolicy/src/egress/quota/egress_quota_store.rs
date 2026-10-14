use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps egress quotas by code.
pub trait EgressQuotaStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EgressQuota>;
    fn save(&self, item: &EgressQuota);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EgressQuota>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEgressQuotaStore {
    pub items: RefCell<BTreeMap<String, EgressQuota>>,
    pub order: RefCell<Vec<String>>,
}

impl EgressQuotaStore for MemoryEgressQuotaStore {
    fn find(&self, code: &str) -> Option<EgressQuota> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EgressQuota) {
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

    fn list(&self) -> Vec<EgressQuota> {
        let mut out: Vec<EgressQuota> = Vec::new();
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

pub fn new_memory_egress_quota_store() -> Rc<dyn EgressQuotaStore> {
    Rc::new(MemoryEgressQuotaStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEgressQuotaStore {
    pub inner: Rc<dyn EgressQuotaStore>,
    pub cache: RefCell<cache::TtlCache<EgressQuota>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EgressQuotaStore for CachedEgressQuotaStore {
    fn find(&self, code: &str) -> Option<EgressQuota> {
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

    fn save(&self, item: &EgressQuota) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EgressQuota> {
        self.inner.list()
    }
}

pub fn new_cached_egress_quota_store(
    inner: Rc<dyn EgressQuotaStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EgressQuotaStore> {
    Rc::new(CachedEgressQuotaStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EgressQuota>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects egress quotas. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressQuotaQuery {
    pub cluster: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EgressQuotaQuery {
    pub fn matches(&self, item: &EgressQuota) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[EgressQuota]) -> Vec<EgressQuota> {
        let matched = collections::filter(items, |candidate: &EgressQuota| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_egress_quotas(store: Rc<dyn EgressQuotaStore>, query: &EgressQuotaQuery) -> Vec<EgressQuota> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn EgressQuotaStore> = new_memory_egress_quota_store();
        let item = new_egress_quota("code-455", "label-959");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
