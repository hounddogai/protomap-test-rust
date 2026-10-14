use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted trust domain baselines by code.
pub trait WeightedTrustDomainBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedTrustDomainBaseline>;
    fn save(&self, item: &WeightedTrustDomainBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedTrustDomainBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedTrustDomainBaselineStore {
    pub items: RefCell<BTreeMap<String, WeightedTrustDomainBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedTrustDomainBaselineStore for MemoryWeightedTrustDomainBaselineStore {
    fn find(&self, code: &str) -> Option<WeightedTrustDomainBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedTrustDomainBaseline) {
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

    fn list(&self) -> Vec<WeightedTrustDomainBaseline> {
        let mut out: Vec<WeightedTrustDomainBaseline> = Vec::new();
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

pub fn new_memory_weighted_trust_domain_baseline_store() -> Rc<dyn WeightedTrustDomainBaselineStore> {
    Rc::new(MemoryWeightedTrustDomainBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedTrustDomainBaselineStore {
    pub inner: Rc<dyn WeightedTrustDomainBaselineStore>,
    pub cache: RefCell<cache::TtlCache<WeightedTrustDomainBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedTrustDomainBaselineStore for CachedWeightedTrustDomainBaselineStore {
    fn find(&self, code: &str) -> Option<WeightedTrustDomainBaseline> {
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

    fn save(&self, item: &WeightedTrustDomainBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedTrustDomainBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_trust_domain_baseline_store(
    inner: Rc<dyn WeightedTrustDomainBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedTrustDomainBaselineStore> {
    Rc::new(CachedWeightedTrustDomainBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedTrustDomainBaseline>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted trust domain baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainBaselineQuery {
    pub namespace: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedTrustDomainBaselineQuery {
    pub fn matches(&self, item: &WeightedTrustDomainBaseline) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedTrustDomainBaseline]) -> Vec<WeightedTrustDomainBaseline> {
        let matched = collections::filter(items, |candidate: &WeightedTrustDomainBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_trust_domain_baselines(
    store: Rc<dyn WeightedTrustDomainBaselineStore>,
    query: &WeightedTrustDomainBaselineQuery,
) -> Vec<WeightedTrustDomainBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedTrustDomainBaselineStore> = new_memory_weighted_trust_domain_baseline_store();
        let item = new_weighted_trust_domain_baseline("code-12", "name-138");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
