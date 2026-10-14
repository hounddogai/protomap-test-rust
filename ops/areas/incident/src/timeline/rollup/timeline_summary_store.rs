use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps timeline summaries by code.
pub trait TimelineSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TimelineSummary>;
    fn save(&self, item: &TimelineSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TimelineSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTimelineSummaryStore {
    pub items: RefCell<BTreeMap<String, TimelineSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl TimelineSummaryStore for MemoryTimelineSummaryStore {
    fn find(&self, code: &str) -> Option<TimelineSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TimelineSummary) {
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

    fn list(&self) -> Vec<TimelineSummary> {
        let mut out: Vec<TimelineSummary> = Vec::new();
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

pub fn new_memory_timeline_summary_store() -> Rc<dyn TimelineSummaryStore> {
    Rc::new(MemoryTimelineSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTimelineSummaryStore {
    pub inner: Rc<dyn TimelineSummaryStore>,
    pub cache: RefCell<cache::TtlCache<TimelineSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TimelineSummaryStore for CachedTimelineSummaryStore {
    fn find(&self, code: &str) -> Option<TimelineSummary> {
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

    fn save(&self, item: &TimelineSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TimelineSummary> {
        self.inner.list()
    }
}

pub fn new_cached_timeline_summary_store(
    inner: Rc<dyn TimelineSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TimelineSummaryStore> {
    Rc::new(CachedTimelineSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TimelineSummary>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects timeline summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSummaryQuery {
    pub namespace: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TimelineSummaryQuery {
    pub fn matches(&self, item: &TimelineSummary) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.attempts < self.min_attempts {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TimelineSummary]) -> Vec<TimelineSummary> {
        let matched = collections::filter(items, |candidate: &TimelineSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_timeline_summaries(
    store: Rc<dyn TimelineSummaryStore>,
    query: &TimelineSummaryQuery,
) -> Vec<TimelineSummary> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TimelineSummaryStore> = new_memory_timeline_summary_store();
        let item = new_timeline_summary("code-768", "title-419");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
