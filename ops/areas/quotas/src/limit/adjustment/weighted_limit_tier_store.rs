use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted limit tiers by code.
pub trait WeightedLimitTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedLimitTier>;
    fn save(&self, item: &WeightedLimitTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedLimitTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedLimitTierStore {
    pub items: RefCell<BTreeMap<String, WeightedLimitTier>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedLimitTierStore for MemoryWeightedLimitTierStore {
    fn find(&self, code: &str) -> Option<WeightedLimitTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedLimitTier) {
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

    fn list(&self) -> Vec<WeightedLimitTier> {
        let mut out: Vec<WeightedLimitTier> = Vec::new();
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

pub fn new_memory_weighted_limit_tier_store() -> Rc<dyn WeightedLimitTierStore> {
    Rc::new(MemoryWeightedLimitTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedLimitTierStore {
    pub inner: Rc<dyn WeightedLimitTierStore>,
    pub cache: RefCell<cache::TtlCache<WeightedLimitTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedLimitTierStore for CachedWeightedLimitTierStore {
    fn find(&self, code: &str) -> Option<WeightedLimitTier> {
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

    fn save(&self, item: &WeightedLimitTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedLimitTier> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_limit_tier_store(
    inner: Rc<dyn WeightedLimitTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedLimitTierStore> {
    Rc::new(CachedWeightedLimitTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedLimitTier>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted limit tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedLimitTierQuery {
    pub country: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedLimitTierQuery {
    pub fn matches(&self, item: &WeightedLimitTier) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
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

    pub fn apply(&self, items: &[WeightedLimitTier]) -> Vec<WeightedLimitTier> {
        let matched = collections::filter(items, |candidate: &WeightedLimitTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_limit_tiers(
    store: Rc<dyn WeightedLimitTierStore>,
    query: &WeightedLimitTierQuery,
) -> Vec<WeightedLimitTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedLimitTierStore> = new_memory_weighted_limit_tier_store();
        let item = new_weighted_limit_tier("code-921", "name-264");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
