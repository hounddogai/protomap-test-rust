use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling peering tiers by code.
pub trait RollingPeeringTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingPeeringTier>;
    fn save(&self, item: &RollingPeeringTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingPeeringTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingPeeringTierStore {
    pub items: RefCell<BTreeMap<String, RollingPeeringTier>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingPeeringTierStore for MemoryRollingPeeringTierStore {
    fn find(&self, code: &str) -> Option<RollingPeeringTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingPeeringTier) {
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

    fn list(&self) -> Vec<RollingPeeringTier> {
        let mut out: Vec<RollingPeeringTier> = Vec::new();
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

pub fn new_memory_rolling_peering_tier_store() -> Rc<dyn RollingPeeringTierStore> {
    Rc::new(MemoryRollingPeeringTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingPeeringTierStore {
    pub inner: Rc<dyn RollingPeeringTierStore>,
    pub cache: RefCell<cache::TtlCache<RollingPeeringTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingPeeringTierStore for CachedRollingPeeringTierStore {
    fn find(&self, code: &str) -> Option<RollingPeeringTier> {
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

    fn save(&self, item: &RollingPeeringTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingPeeringTier> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_peering_tier_store(
    inner: Rc<dyn RollingPeeringTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingPeeringTierStore> {
    Rc::new(CachedRollingPeeringTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingPeeringTier>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling peering tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringTierQuery {
    pub locale: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingPeeringTierQuery {
    pub fn matches(&self, item: &RollingPeeringTier) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingPeeringTier]) -> Vec<RollingPeeringTier> {
        let matched = collections::filter(items, |candidate: &RollingPeeringTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_peering_tiers(
    store: Rc<dyn RollingPeeringTierStore>,
    query: &RollingPeeringTierQuery,
) -> Vec<RollingPeeringTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingPeeringTierStore> = new_memory_rolling_peering_tier_store();
        let item = new_rolling_peering_tier("code-358", "name-782");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
