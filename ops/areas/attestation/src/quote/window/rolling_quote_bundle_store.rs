use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling quote bundles by code.
pub trait RollingQuoteBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingQuoteBundle>;
    fn save(&self, item: &RollingQuoteBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingQuoteBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingQuoteBundleStore {
    pub items: RefCell<BTreeMap<String, RollingQuoteBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingQuoteBundleStore for MemoryRollingQuoteBundleStore {
    fn find(&self, code: &str) -> Option<RollingQuoteBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingQuoteBundle) {
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

    fn list(&self) -> Vec<RollingQuoteBundle> {
        let mut out: Vec<RollingQuoteBundle> = Vec::new();
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

pub fn new_memory_rolling_quote_bundle_store() -> Rc<dyn RollingQuoteBundleStore> {
    Rc::new(MemoryRollingQuoteBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingQuoteBundleStore {
    pub inner: Rc<dyn RollingQuoteBundleStore>,
    pub cache: RefCell<cache::TtlCache<RollingQuoteBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingQuoteBundleStore for CachedRollingQuoteBundleStore {
    fn find(&self, code: &str) -> Option<RollingQuoteBundle> {
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

    fn save(&self, item: &RollingQuoteBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingQuoteBundle> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_quote_bundle_store(
    inner: Rc<dyn RollingQuoteBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingQuoteBundleStore> {
    Rc::new(CachedRollingQuoteBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingQuoteBundle>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling quote bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteBundleQuery {
    pub name: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingQuoteBundleQuery {
    pub fn matches(&self, item: &RollingQuoteBundle) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingQuoteBundle]) -> Vec<RollingQuoteBundle> {
        let matched = collections::filter(items, |candidate: &RollingQuoteBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_quote_bundles(
    store: Rc<dyn RollingQuoteBundleStore>,
    query: &RollingQuoteBundleQuery,
) -> Vec<RollingQuoteBundle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingQuoteBundleStore> = new_memory_rolling_quote_bundle_store();
        let item = new_rolling_quote_bundle("code-314", "name-273");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
