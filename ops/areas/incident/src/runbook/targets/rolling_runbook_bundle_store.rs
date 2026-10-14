use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling runbook bundles by code.
pub trait RollingRunbookBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingRunbookBundle>;
    fn save(&self, item: &RollingRunbookBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingRunbookBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingRunbookBundleStore {
    pub items: RefCell<BTreeMap<String, RollingRunbookBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingRunbookBundleStore for MemoryRollingRunbookBundleStore {
    fn find(&self, code: &str) -> Option<RollingRunbookBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingRunbookBundle) {
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

    fn list(&self) -> Vec<RollingRunbookBundle> {
        let mut out: Vec<RollingRunbookBundle> = Vec::new();
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

pub fn new_memory_rolling_runbook_bundle_store() -> Rc<dyn RollingRunbookBundleStore> {
    Rc::new(MemoryRollingRunbookBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingRunbookBundleStore {
    pub inner: Rc<dyn RollingRunbookBundleStore>,
    pub cache: RefCell<cache::TtlCache<RollingRunbookBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingRunbookBundleStore for CachedRollingRunbookBundleStore {
    fn find(&self, code: &str) -> Option<RollingRunbookBundle> {
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

    fn save(&self, item: &RollingRunbookBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingRunbookBundle> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_runbook_bundle_store(
    inner: Rc<dyn RollingRunbookBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingRunbookBundleStore> {
    Rc::new(CachedRollingRunbookBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingRunbookBundle>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling runbook bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookBundleQuery {
    pub segment: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingRunbookBundleQuery {
    pub fn matches(&self, item: &RollingRunbookBundle) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingRunbookBundle]) -> Vec<RollingRunbookBundle> {
        let matched = collections::filter(items, |candidate: &RollingRunbookBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_runbook_bundles(
    store: Rc<dyn RollingRunbookBundleStore>,
    query: &RollingRunbookBundleQuery,
) -> Vec<RollingRunbookBundle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingRunbookBundleStore> = new_memory_rolling_runbook_bundle_store();
        let item = new_rolling_runbook_bundle("code-946", "label-967");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
