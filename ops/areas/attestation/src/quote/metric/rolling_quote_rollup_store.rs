use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling quote rollups by code.
pub trait RollingQuoteRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingQuoteRollup>;
    fn save(&self, item: &RollingQuoteRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingQuoteRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingQuoteRollupStore {
    pub items: RefCell<BTreeMap<String, RollingQuoteRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingQuoteRollupStore for MemoryRollingQuoteRollupStore {
    fn find(&self, code: &str) -> Option<RollingQuoteRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingQuoteRollup) {
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

    fn list(&self) -> Vec<RollingQuoteRollup> {
        let mut out: Vec<RollingQuoteRollup> = Vec::new();
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

pub fn new_memory_rolling_quote_rollup_store() -> Rc<dyn RollingQuoteRollupStore> {
    Rc::new(MemoryRollingQuoteRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingQuoteRollupStore {
    pub inner: Rc<dyn RollingQuoteRollupStore>,
    pub cache: RefCell<cache::TtlCache<RollingQuoteRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingQuoteRollupStore for CachedRollingQuoteRollupStore {
    fn find(&self, code: &str) -> Option<RollingQuoteRollup> {
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

    fn save(&self, item: &RollingQuoteRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingQuoteRollup> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_quote_rollup_store(
    inner: Rc<dyn RollingQuoteRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingQuoteRollupStore> {
    Rc::new(CachedRollingQuoteRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingQuoteRollup>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling quote rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteRollupQuery {
    pub cluster: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingQuoteRollupQuery {
    pub fn matches(&self, item: &RollingQuoteRollup) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingQuoteRollup]) -> Vec<RollingQuoteRollup> {
        let matched = collections::filter(items, |candidate: &RollingQuoteRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_quote_rollups(
    store: Rc<dyn RollingQuoteRollupStore>,
    query: &RollingQuoteRollupQuery,
) -> Vec<RollingQuoteRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingQuoteRollupStore> = new_memory_rolling_quote_rollup_store();
        let item = new_rolling_quote_rollup("code-604", "title-541");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
