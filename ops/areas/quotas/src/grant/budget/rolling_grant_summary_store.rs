use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling grant summaries by code.
pub trait RollingGrantSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingGrantSummary>;
    fn save(&self, item: &RollingGrantSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingGrantSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingGrantSummaryStore {
    pub items: RefCell<BTreeMap<String, RollingGrantSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingGrantSummaryStore for MemoryRollingGrantSummaryStore {
    fn find(&self, code: &str) -> Option<RollingGrantSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingGrantSummary) {
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

    fn list(&self) -> Vec<RollingGrantSummary> {
        let mut out: Vec<RollingGrantSummary> = Vec::new();
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

pub fn new_memory_rolling_grant_summary_store() -> Rc<dyn RollingGrantSummaryStore> {
    Rc::new(MemoryRollingGrantSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingGrantSummaryStore {
    pub inner: Rc<dyn RollingGrantSummaryStore>,
    pub cache: RefCell<cache::TtlCache<RollingGrantSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingGrantSummaryStore for CachedRollingGrantSummaryStore {
    fn find(&self, code: &str) -> Option<RollingGrantSummary> {
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

    fn save(&self, item: &RollingGrantSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingGrantSummary> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_grant_summary_store(
    inner: Rc<dyn RollingGrantSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingGrantSummaryStore> {
    Rc::new(CachedRollingGrantSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingGrantSummary>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling grant summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantSummaryQuery {
    pub title: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingGrantSummaryQuery {
    pub fn matches(&self, item: &RollingGrantSummary) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingGrantSummary]) -> Vec<RollingGrantSummary> {
        let matched = collections::filter(items, |candidate: &RollingGrantSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_grant_summaries(
    store: Rc<dyn RollingGrantSummaryStore>,
    query: &RollingGrantSummaryQuery,
) -> Vec<RollingGrantSummary> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingGrantSummaryStore> = new_memory_rolling_grant_summary_store();
        let item = new_rolling_grant_summary("code-562", "title-402");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
