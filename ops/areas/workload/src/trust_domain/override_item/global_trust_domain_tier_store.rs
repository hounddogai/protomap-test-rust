use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global trust domain tiers by code.
pub trait GlobalTrustDomainTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalTrustDomainTier>;
    fn save(&self, item: &GlobalTrustDomainTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalTrustDomainTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalTrustDomainTierStore {
    pub items: RefCell<BTreeMap<String, GlobalTrustDomainTier>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalTrustDomainTierStore for MemoryGlobalTrustDomainTierStore {
    fn find(&self, code: &str) -> Option<GlobalTrustDomainTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalTrustDomainTier) {
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

    fn list(&self) -> Vec<GlobalTrustDomainTier> {
        let mut out: Vec<GlobalTrustDomainTier> = Vec::new();
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

pub fn new_memory_global_trust_domain_tier_store() -> Rc<dyn GlobalTrustDomainTierStore> {
    Rc::new(MemoryGlobalTrustDomainTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalTrustDomainTierStore {
    pub inner: Rc<dyn GlobalTrustDomainTierStore>,
    pub cache: RefCell<cache::TtlCache<GlobalTrustDomainTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalTrustDomainTierStore for CachedGlobalTrustDomainTierStore {
    fn find(&self, code: &str) -> Option<GlobalTrustDomainTier> {
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

    fn save(&self, item: &GlobalTrustDomainTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalTrustDomainTier> {
        self.inner.list()
    }
}

pub fn new_cached_global_trust_domain_tier_store(
    inner: Rc<dyn GlobalTrustDomainTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalTrustDomainTierStore> {
    Rc::new(CachedGlobalTrustDomainTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalTrustDomainTier>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global trust domain tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTrustDomainTierQuery {
    pub name: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalTrustDomainTierQuery {
    pub fn matches(&self, item: &GlobalTrustDomainTier) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalTrustDomainTier]) -> Vec<GlobalTrustDomainTier> {
        let matched = collections::filter(items, |candidate: &GlobalTrustDomainTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_trust_domain_tiers(
    store: Rc<dyn GlobalTrustDomainTierStore>,
    query: &GlobalTrustDomainTierQuery,
) -> Vec<GlobalTrustDomainTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalTrustDomainTierStore> = new_memory_global_trust_domain_tier_store();
        let item = new_global_trust_domain_tier("code-379", "name-32");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
