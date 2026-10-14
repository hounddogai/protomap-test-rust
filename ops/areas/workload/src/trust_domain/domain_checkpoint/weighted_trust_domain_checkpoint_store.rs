use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted trust domain checkpoints by code.
pub trait WeightedTrustDomainCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedTrustDomainCheckpoint>;
    fn save(&self, item: &WeightedTrustDomainCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedTrustDomainCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedTrustDomainCheckpointStore {
    pub items: RefCell<BTreeMap<String, WeightedTrustDomainCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedTrustDomainCheckpointStore for MemoryWeightedTrustDomainCheckpointStore {
    fn find(&self, code: &str) -> Option<WeightedTrustDomainCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedTrustDomainCheckpoint) {
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

    fn list(&self) -> Vec<WeightedTrustDomainCheckpoint> {
        let mut out: Vec<WeightedTrustDomainCheckpoint> = Vec::new();
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

pub fn new_memory_weighted_trust_domain_checkpoint_store() -> Rc<dyn WeightedTrustDomainCheckpointStore> {
    Rc::new(MemoryWeightedTrustDomainCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedTrustDomainCheckpointStore {
    pub inner: Rc<dyn WeightedTrustDomainCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<WeightedTrustDomainCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedTrustDomainCheckpointStore for CachedWeightedTrustDomainCheckpointStore {
    fn find(&self, code: &str) -> Option<WeightedTrustDomainCheckpoint> {
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

    fn save(&self, item: &WeightedTrustDomainCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedTrustDomainCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_trust_domain_checkpoint_store(
    inner: Rc<dyn WeightedTrustDomainCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedTrustDomainCheckpointStore> {
    Rc::new(CachedWeightedTrustDomainCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedTrustDomainCheckpoint>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted trust domain checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainCheckpointQuery {
    pub country: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedTrustDomainCheckpointQuery {
    pub fn matches(&self, item: &WeightedTrustDomainCheckpoint) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
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

    pub fn apply(&self, items: &[WeightedTrustDomainCheckpoint]) -> Vec<WeightedTrustDomainCheckpoint> {
        let matched = collections::filter(items, |candidate: &WeightedTrustDomainCheckpoint| {
            self.matches(candidate)
        });
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_trust_domain_checkpoints(
    store: Rc<dyn WeightedTrustDomainCheckpointStore>,
    query: &WeightedTrustDomainCheckpointQuery,
) -> Vec<WeightedTrustDomainCheckpoint> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedTrustDomainCheckpointStore> = new_memory_weighted_trust_domain_checkpoint_store();
        let item = new_weighted_trust_domain_checkpoint("code-374", "title-25");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
