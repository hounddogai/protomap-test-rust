use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global peering tiers by code.
pub trait GlobalPeeringTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalPeeringTier>;
    fn save(&self, item: &GlobalPeeringTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalPeeringTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalPeeringTierStore {
    pub items: RefCell<BTreeMap<String, GlobalPeeringTier>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalPeeringTierStore for MemoryGlobalPeeringTierStore {
    fn find(&self, code: &str) -> Option<GlobalPeeringTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalPeeringTier) {
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

    fn list(&self) -> Vec<GlobalPeeringTier> {
        let mut out: Vec<GlobalPeeringTier> = Vec::new();
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

pub fn new_memory_global_peering_tier_store() -> Rc<dyn GlobalPeeringTierStore> {
    Rc::new(MemoryGlobalPeeringTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalPeeringTierStore {
    pub inner: Rc<dyn GlobalPeeringTierStore>,
    pub cache: RefCell<cache::TtlCache<GlobalPeeringTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalPeeringTierStore for CachedGlobalPeeringTierStore {
    fn find(&self, code: &str) -> Option<GlobalPeeringTier> {
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

    fn save(&self, item: &GlobalPeeringTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalPeeringTier> {
        self.inner.list()
    }
}

pub fn new_cached_global_peering_tier_store(
    inner: Rc<dyn GlobalPeeringTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalPeeringTierStore> {
    Rc::new(CachedGlobalPeeringTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalPeeringTier>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global peering tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringTierQuery {
    pub title: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalPeeringTierQuery {
    pub fn matches(&self, item: &GlobalPeeringTier) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalPeeringTier]) -> Vec<GlobalPeeringTier> {
        let matched = collections::filter(items, |candidate: &GlobalPeeringTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_peering_tiers(
    store: Rc<dyn GlobalPeeringTierStore>,
    query: &GlobalPeeringTierQuery,
) -> Vec<GlobalPeeringTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalPeeringTierStore> = new_memory_global_peering_tier_store();
        let item = new_global_peering_tier("code-464", "title-724");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
