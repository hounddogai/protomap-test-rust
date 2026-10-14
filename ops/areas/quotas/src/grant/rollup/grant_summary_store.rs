use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps grant summaries by code.
pub trait GrantSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GrantSummary>;
    fn save(&self, item: &GrantSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GrantSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGrantSummaryStore {
    pub items: RefCell<BTreeMap<String, GrantSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl GrantSummaryStore for MemoryGrantSummaryStore {
    fn find(&self, code: &str) -> Option<GrantSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GrantSummary) {
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

    fn list(&self) -> Vec<GrantSummary> {
        let mut out: Vec<GrantSummary> = Vec::new();
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

pub fn new_memory_grant_summary_store() -> Rc<dyn GrantSummaryStore> {
    Rc::new(MemoryGrantSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGrantSummaryStore {
    pub inner: Rc<dyn GrantSummaryStore>,
    pub cache: RefCell<cache::TtlCache<GrantSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GrantSummaryStore for CachedGrantSummaryStore {
    fn find(&self, code: &str) -> Option<GrantSummary> {
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

    fn save(&self, item: &GrantSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GrantSummary> {
        self.inner.list()
    }
}

pub fn new_cached_grant_summary_store(
    inner: Rc<dyn GrantSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GrantSummaryStore> {
    Rc::new(CachedGrantSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GrantSummary>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects grant summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantSummaryQuery {
    pub category: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GrantSummaryQuery {
    pub fn matches(&self, item: &GrantSummary) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GrantSummary]) -> Vec<GrantSummary> {
        let matched = collections::filter(items, |candidate: &GrantSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_grant_summaries(store: Rc<dyn GrantSummaryStore>, query: &GrantSummaryQuery) -> Vec<GrantSummary> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GrantSummaryStore> = new_memory_grant_summary_store();
        let item = new_grant_summary("code-962", "label-254");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
