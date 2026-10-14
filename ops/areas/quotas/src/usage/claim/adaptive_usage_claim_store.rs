use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive usage claims by code.
pub trait AdaptiveUsageClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveUsageClaim>;
    fn save(&self, item: &AdaptiveUsageClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveUsageClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveUsageClaimStore {
    pub items: RefCell<BTreeMap<String, AdaptiveUsageClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveUsageClaimStore for MemoryAdaptiveUsageClaimStore {
    fn find(&self, code: &str) -> Option<AdaptiveUsageClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveUsageClaim) {
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

    fn list(&self) -> Vec<AdaptiveUsageClaim> {
        let mut out: Vec<AdaptiveUsageClaim> = Vec::new();
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

pub fn new_memory_adaptive_usage_claim_store() -> Rc<dyn AdaptiveUsageClaimStore> {
    Rc::new(MemoryAdaptiveUsageClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveUsageClaimStore {
    pub inner: Rc<dyn AdaptiveUsageClaimStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveUsageClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveUsageClaimStore for CachedAdaptiveUsageClaimStore {
    fn find(&self, code: &str) -> Option<AdaptiveUsageClaim> {
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

    fn save(&self, item: &AdaptiveUsageClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveUsageClaim> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_usage_claim_store(
    inner: Rc<dyn AdaptiveUsageClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveUsageClaimStore> {
    Rc::new(CachedAdaptiveUsageClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveUsageClaim>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive usage claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageClaimQuery {
    pub segment: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveUsageClaimQuery {
    pub fn matches(&self, item: &AdaptiveUsageClaim) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
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

    pub fn apply(&self, items: &[AdaptiveUsageClaim]) -> Vec<AdaptiveUsageClaim> {
        let matched = collections::filter(items, |candidate: &AdaptiveUsageClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_usage_claims(
    store: Rc<dyn AdaptiveUsageClaimStore>,
    query: &AdaptiveUsageClaimQuery,
) -> Vec<AdaptiveUsageClaim> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AdaptiveUsageClaimStore> = new_memory_adaptive_usage_claim_store();
        let item = new_adaptive_usage_claim("code-378", "name-953");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
