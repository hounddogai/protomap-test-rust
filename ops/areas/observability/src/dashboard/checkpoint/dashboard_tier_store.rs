use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dashboard tiers by code.
pub trait DashboardTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DashboardTier>;
    fn save(&self, item: &DashboardTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DashboardTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDashboardTierStore {
    pub items: RefCell<BTreeMap<String, DashboardTier>>,
    pub order: RefCell<Vec<String>>,
}

impl DashboardTierStore for MemoryDashboardTierStore {
    fn find(&self, code: &str) -> Option<DashboardTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DashboardTier) {
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

    fn list(&self) -> Vec<DashboardTier> {
        let mut out: Vec<DashboardTier> = Vec::new();
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

pub fn new_memory_dashboard_tier_store() -> Rc<dyn DashboardTierStore> {
    Rc::new(MemoryDashboardTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDashboardTierStore {
    pub inner: Rc<dyn DashboardTierStore>,
    pub cache: RefCell<cache::TtlCache<DashboardTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DashboardTierStore for CachedDashboardTierStore {
    fn find(&self, code: &str) -> Option<DashboardTier> {
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

    fn save(&self, item: &DashboardTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DashboardTier> {
        self.inner.list()
    }
}

pub fn new_cached_dashboard_tier_store(
    inner: Rc<dyn DashboardTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DashboardTierStore> {
    Rc::new(CachedDashboardTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DashboardTier>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dashboard tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTierQuery {
    pub channel: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DashboardTierQuery {
    pub fn matches(&self, item: &DashboardTier) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DashboardTier]) -> Vec<DashboardTier> {
        let matched = collections::filter(items, |candidate: &DashboardTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dashboard_tiers(store: Rc<dyn DashboardTierStore>, query: &DashboardTierQuery) -> Vec<DashboardTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DashboardTierStore> = new_memory_dashboard_tier_store();
        let item = new_dashboard_tier("code-42", "title-672");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
