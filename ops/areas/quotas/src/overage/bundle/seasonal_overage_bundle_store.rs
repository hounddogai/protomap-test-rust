use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal overage bundles by code.
pub trait SeasonalOverageBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalOverageBundle>;
    fn save(&self, item: &SeasonalOverageBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalOverageBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalOverageBundleStore {
    pub items: RefCell<BTreeMap<String, SeasonalOverageBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalOverageBundleStore for MemorySeasonalOverageBundleStore {
    fn find(&self, code: &str) -> Option<SeasonalOverageBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalOverageBundle) {
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

    fn list(&self) -> Vec<SeasonalOverageBundle> {
        let mut out: Vec<SeasonalOverageBundle> = Vec::new();
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

pub fn new_memory_seasonal_overage_bundle_store() -> Rc<dyn SeasonalOverageBundleStore> {
    Rc::new(MemorySeasonalOverageBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalOverageBundleStore {
    pub inner: Rc<dyn SeasonalOverageBundleStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalOverageBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalOverageBundleStore for CachedSeasonalOverageBundleStore {
    fn find(&self, code: &str) -> Option<SeasonalOverageBundle> {
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

    fn save(&self, item: &SeasonalOverageBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalOverageBundle> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_overage_bundle_store(
    inner: Rc<dyn SeasonalOverageBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalOverageBundleStore> {
    Rc::new(CachedSeasonalOverageBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalOverageBundle>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal overage bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalOverageBundleQuery {
    pub scope: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalOverageBundleQuery {
    pub fn matches(&self, item: &SeasonalOverageBundle) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
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

    pub fn apply(&self, items: &[SeasonalOverageBundle]) -> Vec<SeasonalOverageBundle> {
        let matched = collections::filter(items, |candidate: &SeasonalOverageBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_overage_bundles(
    store: Rc<dyn SeasonalOverageBundleStore>,
    query: &SeasonalOverageBundleQuery,
) -> Vec<SeasonalOverageBundle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalOverageBundleStore> = new_memory_seasonal_overage_bundle_store();
        let item = new_seasonal_overage_bundle("code-939", "title-421");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
