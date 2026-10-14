use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient health check summaries by code.
pub trait LenientHealthCheckSummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientHealthCheckSummary>;
    fn save(&self, item: &LenientHealthCheckSummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientHealthCheckSummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientHealthCheckSummaryStore {
    pub items: RefCell<BTreeMap<String, LenientHealthCheckSummary>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientHealthCheckSummaryStore for MemoryLenientHealthCheckSummaryStore {
    fn find(&self, code: &str) -> Option<LenientHealthCheckSummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientHealthCheckSummary) {
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

    fn list(&self) -> Vec<LenientHealthCheckSummary> {
        let mut out: Vec<LenientHealthCheckSummary> = Vec::new();
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

pub fn new_memory_lenient_health_check_summary_store() -> Rc<dyn LenientHealthCheckSummaryStore> {
    Rc::new(MemoryLenientHealthCheckSummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientHealthCheckSummaryStore {
    pub inner: Rc<dyn LenientHealthCheckSummaryStore>,
    pub cache: RefCell<cache::TtlCache<LenientHealthCheckSummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientHealthCheckSummaryStore for CachedLenientHealthCheckSummaryStore {
    fn find(&self, code: &str) -> Option<LenientHealthCheckSummary> {
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

    fn save(&self, item: &LenientHealthCheckSummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientHealthCheckSummary> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_health_check_summary_store(
    inner: Rc<dyn LenientHealthCheckSummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientHealthCheckSummaryStore> {
    Rc::new(CachedLenientHealthCheckSummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientHealthCheckSummary>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient health check summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientHealthCheckSummaryQuery {
    pub country: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientHealthCheckSummaryQuery {
    pub fn matches(&self, item: &LenientHealthCheckSummary) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientHealthCheckSummary]) -> Vec<LenientHealthCheckSummary> {
        let matched = collections::filter(items, |candidate: &LenientHealthCheckSummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_health_check_summaries(
    store: Rc<dyn LenientHealthCheckSummaryStore>,
    query: &LenientHealthCheckSummaryQuery,
) -> Vec<LenientHealthCheckSummary> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientHealthCheckSummaryStore> = new_memory_lenient_health_check_summary_store();
        let item = new_lenient_health_check_summary("code-361", "name-338");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
