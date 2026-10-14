use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps release summaries by code.
pub trait ReleaseSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ReleaseSummary>;
    fn save(&self, item: &ReleaseSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ReleaseSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryReleaseSummaryStore {
    pub items: RefCell<BTreeMap<String, ReleaseSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl ReleaseSummaryStore for MemoryReleaseSummaryStore {
    fn find(&self, code: &str) -> Option<ReleaseSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ReleaseSummary) {
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

    fn list(&self) -> Vec<ReleaseSummary> {
        let mut out: Vec<ReleaseSummary> = Vec::new();
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

pub fn new_memory_release_summary_store() -> Rc<dyn ReleaseSummaryStore> {
    Rc::new(MemoryReleaseSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedReleaseSummaryStore {
    pub inner: Rc<dyn ReleaseSummaryStore>,
    pub cache: RefCell<cache::TtlCache<ReleaseSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ReleaseSummaryStore for CachedReleaseSummaryStore {
    fn find(&self, code: &str) -> Option<ReleaseSummary> {
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

    fn save(&self, item: &ReleaseSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ReleaseSummary> {
        self.inner.list()
    }
}

pub fn new_cached_release_summary_store(
    inner: Rc<dyn ReleaseSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ReleaseSummaryStore> {
    Rc::new(CachedReleaseSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ReleaseSummary>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects release summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSummaryQuery {
    pub scope: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ReleaseSummaryQuery {
    pub fn matches(&self, item: &ReleaseSummary) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ReleaseSummary]) -> Vec<ReleaseSummary> {
        let matched = collections::filter(items, |candidate: &ReleaseSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_release_summaries(store: Rc<dyn ReleaseSummaryStore>, query: &ReleaseSummaryQuery) -> Vec<ReleaseSummary> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ReleaseSummaryStore> = new_memory_release_summary_store();
        let item = new_release_summary("code-15", "name-37");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
