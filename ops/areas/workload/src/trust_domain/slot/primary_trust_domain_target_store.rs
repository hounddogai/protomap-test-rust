use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary trust domain targets by code.
pub trait PrimaryTrustDomainTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryTrustDomainTarget>;
    fn save(&self, item: &PrimaryTrustDomainTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryTrustDomainTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryTrustDomainTargetStore {
    pub items: RefCell<BTreeMap<String, PrimaryTrustDomainTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryTrustDomainTargetStore for MemoryPrimaryTrustDomainTargetStore {
    fn find(&self, code: &str) -> Option<PrimaryTrustDomainTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryTrustDomainTarget) {
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

    fn list(&self) -> Vec<PrimaryTrustDomainTarget> {
        let mut out: Vec<PrimaryTrustDomainTarget> = Vec::new();
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

pub fn new_memory_primary_trust_domain_target_store() -> Rc<dyn PrimaryTrustDomainTargetStore> {
    Rc::new(MemoryPrimaryTrustDomainTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryTrustDomainTargetStore {
    pub inner: Rc<dyn PrimaryTrustDomainTargetStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryTrustDomainTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryTrustDomainTargetStore for CachedPrimaryTrustDomainTargetStore {
    fn find(&self, code: &str) -> Option<PrimaryTrustDomainTarget> {
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

    fn save(&self, item: &PrimaryTrustDomainTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryTrustDomainTarget> {
        self.inner.list()
    }
}

pub fn new_cached_primary_trust_domain_target_store(
    inner: Rc<dyn PrimaryTrustDomainTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryTrustDomainTargetStore> {
    Rc::new(CachedPrimaryTrustDomainTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryTrustDomainTarget>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary trust domain targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTrustDomainTargetQuery {
    pub category: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryTrustDomainTargetQuery {
    pub fn matches(&self, item: &PrimaryTrustDomainTarget) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryTrustDomainTarget]) -> Vec<PrimaryTrustDomainTarget> {
        let matched = collections::filter(items, |candidate: &PrimaryTrustDomainTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_trust_domain_targets(
    store: Rc<dyn PrimaryTrustDomainTargetStore>,
    query: &PrimaryTrustDomainTargetQuery,
) -> Vec<PrimaryTrustDomainTarget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryTrustDomainTargetStore> = new_memory_primary_trust_domain_target_store();
        let item = new_primary_trust_domain_target("code-62", "label-693");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
