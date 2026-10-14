use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped runbook variances by code.
pub trait CappedRunbookVarianceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedRunbookVariance>;
    fn save(&self, item: &CappedRunbookVariance);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedRunbookVariance>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedRunbookVarianceStore {
    pub items: RefCell<BTreeMap<String, CappedRunbookVariance>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedRunbookVarianceStore for MemoryCappedRunbookVarianceStore {
    fn find(&self, code: &str) -> Option<CappedRunbookVariance> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedRunbookVariance) {
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

    fn list(&self) -> Vec<CappedRunbookVariance> {
        let mut out: Vec<CappedRunbookVariance> = Vec::new();
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

pub fn new_memory_capped_runbook_variance_store() -> Rc<dyn CappedRunbookVarianceStore> {
    Rc::new(MemoryCappedRunbookVarianceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedRunbookVarianceStore {
    pub inner: Rc<dyn CappedRunbookVarianceStore>,
    pub cache: RefCell<cache::TtlCache<CappedRunbookVariance>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedRunbookVarianceStore for CachedCappedRunbookVarianceStore {
    fn find(&self, code: &str) -> Option<CappedRunbookVariance> {
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

    fn save(&self, item: &CappedRunbookVariance) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedRunbookVariance> {
        self.inner.list()
    }
}

pub fn new_cached_capped_runbook_variance_store(
    inner: Rc<dyn CappedRunbookVarianceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedRunbookVarianceStore> {
    Rc::new(CachedCappedRunbookVarianceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedRunbookVariance>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped runbook variances. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRunbookVarianceQuery {
    pub cluster: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedRunbookVarianceQuery {
    pub fn matches(&self, item: &CappedRunbookVariance) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedRunbookVariance]) -> Vec<CappedRunbookVariance> {
        let matched = collections::filter(items, |candidate: &CappedRunbookVariance| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_runbook_variances(
    store: Rc<dyn CappedRunbookVarianceStore>,
    query: &CappedRunbookVarianceQuery,
) -> Vec<CappedRunbookVariance> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedRunbookVarianceStore> = new_memory_capped_runbook_variance_store();
        let item = new_capped_runbook_variance("code-448", "label-874");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
