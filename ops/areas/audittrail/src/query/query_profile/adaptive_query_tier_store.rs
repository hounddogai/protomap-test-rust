use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive query tiers by code.
pub trait AdaptiveQueryTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveQueryTier>;
    fn save(&self, item: &AdaptiveQueryTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveQueryTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveQueryTierStore {
    pub items: RefCell<BTreeMap<String, AdaptiveQueryTier>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveQueryTierStore for MemoryAdaptiveQueryTierStore {
    fn find(&self, code: &str) -> Option<AdaptiveQueryTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveQueryTier) {
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

    fn list(&self) -> Vec<AdaptiveQueryTier> {
        let mut out: Vec<AdaptiveQueryTier> = Vec::new();
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

pub fn new_memory_adaptive_query_tier_store() -> Rc<dyn AdaptiveQueryTierStore> {
    Rc::new(MemoryAdaptiveQueryTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveQueryTierStore {
    pub inner: Rc<dyn AdaptiveQueryTierStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveQueryTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveQueryTierStore for CachedAdaptiveQueryTierStore {
    fn find(&self, code: &str) -> Option<AdaptiveQueryTier> {
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

    fn save(&self, item: &AdaptiveQueryTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveQueryTier> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_query_tier_store(
    inner: Rc<dyn AdaptiveQueryTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveQueryTierStore> {
    Rc::new(CachedAdaptiveQueryTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveQueryTier>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive query tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryTierQuery {
    pub locale: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveQueryTierQuery {
    pub fn matches(&self, item: &AdaptiveQueryTier) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
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

    pub fn apply(&self, items: &[AdaptiveQueryTier]) -> Vec<AdaptiveQueryTier> {
        let matched = collections::filter(items, |candidate: &AdaptiveQueryTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_query_tiers(
    store: Rc<dyn AdaptiveQueryTierStore>,
    query: &AdaptiveQueryTierQuery,
) -> Vec<AdaptiveQueryTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AdaptiveQueryTierStore> = new_memory_adaptive_query_tier_store();
        let item = new_adaptive_query_tier("code-310", "label-533");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
