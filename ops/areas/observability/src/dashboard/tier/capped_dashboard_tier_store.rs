use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped dashboard tiers by code.
pub trait CappedDashboardTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedDashboardTier>;
    fn save(&self, item: &CappedDashboardTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedDashboardTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedDashboardTierStore {
    pub items: RefCell<BTreeMap<String, CappedDashboardTier>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedDashboardTierStore for MemoryCappedDashboardTierStore {
    fn find(&self, code: &str) -> Option<CappedDashboardTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedDashboardTier) {
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

    fn list(&self) -> Vec<CappedDashboardTier> {
        let mut out: Vec<CappedDashboardTier> = Vec::new();
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

pub fn new_memory_capped_dashboard_tier_store() -> Rc<dyn CappedDashboardTierStore> {
    Rc::new(MemoryCappedDashboardTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedDashboardTierStore {
    pub inner: Rc<dyn CappedDashboardTierStore>,
    pub cache: RefCell<cache::TtlCache<CappedDashboardTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedDashboardTierStore for CachedCappedDashboardTierStore {
    fn find(&self, code: &str) -> Option<CappedDashboardTier> {
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

    fn save(&self, item: &CappedDashboardTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedDashboardTier> {
        self.inner.list()
    }
}

pub fn new_cached_capped_dashboard_tier_store(
    inner: Rc<dyn CappedDashboardTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedDashboardTierStore> {
    Rc::new(CachedCappedDashboardTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedDashboardTier>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped dashboard tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardTierQuery {
    pub category: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedDashboardTierQuery {
    pub fn matches(&self, item: &CappedDashboardTier) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedDashboardTier]) -> Vec<CappedDashboardTier> {
        let matched = collections::filter(items, |candidate: &CappedDashboardTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_dashboard_tiers(
    store: Rc<dyn CappedDashboardTierStore>,
    query: &CappedDashboardTierQuery,
) -> Vec<CappedDashboardTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedDashboardTierStore> = new_memory_capped_dashboard_tier_store();
        let item = new_capped_dashboard_tier("code-940", "label-988");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
