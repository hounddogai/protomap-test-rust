use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled trust domain tiers by code.
pub trait ScheduledTrustDomainTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledTrustDomainTier>;
    fn save(&self, item: &ScheduledTrustDomainTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledTrustDomainTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledTrustDomainTierStore {
    pub items: RefCell<BTreeMap<String, ScheduledTrustDomainTier>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledTrustDomainTierStore for MemoryScheduledTrustDomainTierStore {
    fn find(&self, code: &str) -> Option<ScheduledTrustDomainTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledTrustDomainTier) {
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

    fn list(&self) -> Vec<ScheduledTrustDomainTier> {
        let mut out: Vec<ScheduledTrustDomainTier> = Vec::new();
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

pub fn new_memory_scheduled_trust_domain_tier_store() -> Rc<dyn ScheduledTrustDomainTierStore> {
    Rc::new(MemoryScheduledTrustDomainTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledTrustDomainTierStore {
    pub inner: Rc<dyn ScheduledTrustDomainTierStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledTrustDomainTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledTrustDomainTierStore for CachedScheduledTrustDomainTierStore {
    fn find(&self, code: &str) -> Option<ScheduledTrustDomainTier> {
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

    fn save(&self, item: &ScheduledTrustDomainTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledTrustDomainTier> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_trust_domain_tier_store(
    inner: Rc<dyn ScheduledTrustDomainTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledTrustDomainTierStore> {
    Rc::new(CachedScheduledTrustDomainTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledTrustDomainTier>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled trust domain tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTrustDomainTierQuery {
    pub source: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledTrustDomainTierQuery {
    pub fn matches(&self, item: &ScheduledTrustDomainTier) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledTrustDomainTier]) -> Vec<ScheduledTrustDomainTier> {
        let matched = collections::filter(items, |candidate: &ScheduledTrustDomainTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_trust_domain_tiers(
    store: Rc<dyn ScheduledTrustDomainTierStore>,
    query: &ScheduledTrustDomainTierQuery,
) -> Vec<ScheduledTrustDomainTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ScheduledTrustDomainTierStore> = new_memory_scheduled_trust_domain_tier_store();
        let item = new_scheduled_trust_domain_tier("code-607", "name-787");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
