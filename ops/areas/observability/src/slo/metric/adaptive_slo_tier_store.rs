use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive slo tiers by code.
pub trait AdaptiveSloTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveSloTier>;
    fn save(&self, item: &AdaptiveSloTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveSloTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveSloTierStore {
    pub items: RefCell<BTreeMap<String, AdaptiveSloTier>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveSloTierStore for MemoryAdaptiveSloTierStore {
    fn find(&self, code: &str) -> Option<AdaptiveSloTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveSloTier) {
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

    fn list(&self) -> Vec<AdaptiveSloTier> {
        let mut out: Vec<AdaptiveSloTier> = Vec::new();
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

pub fn new_memory_adaptive_slo_tier_store() -> Rc<dyn AdaptiveSloTierStore> {
    Rc::new(MemoryAdaptiveSloTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveSloTierStore {
    pub inner: Rc<dyn AdaptiveSloTierStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveSloTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveSloTierStore for CachedAdaptiveSloTierStore {
    fn find(&self, code: &str) -> Option<AdaptiveSloTier> {
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

    fn save(&self, item: &AdaptiveSloTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveSloTier> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_slo_tier_store(
    inner: Rc<dyn AdaptiveSloTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveSloTierStore> {
    Rc::new(CachedAdaptiveSloTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveSloTier>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive slo tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloTierQuery {
    pub namespace: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveSloTierQuery {
    pub fn matches(&self, item: &AdaptiveSloTier) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[AdaptiveSloTier]) -> Vec<AdaptiveSloTier> {
        let matched = collections::filter(items, |candidate: &AdaptiveSloTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_slo_tiers(
    store: Rc<dyn AdaptiveSloTierStore>,
    query: &AdaptiveSloTierQuery,
) -> Vec<AdaptiveSloTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AdaptiveSloTierStore> = new_memory_adaptive_slo_tier_store();
        let item = new_adaptive_slo_tier("code-449", "title-600");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
