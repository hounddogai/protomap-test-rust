use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient postmortem estimates by code.
pub trait LenientPostmortemEstimateStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientPostmortemEstimate>;
    fn save(&self, item: &LenientPostmortemEstimate);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientPostmortemEstimate>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientPostmortemEstimateStore {
    pub items: RefCell<BTreeMap<String, LenientPostmortemEstimate>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientPostmortemEstimateStore for MemoryLenientPostmortemEstimateStore {
    fn find(&self, code: &str) -> Option<LenientPostmortemEstimate> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientPostmortemEstimate) {
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

    fn list(&self) -> Vec<LenientPostmortemEstimate> {
        let mut out: Vec<LenientPostmortemEstimate> = Vec::new();
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

pub fn new_memory_lenient_postmortem_estimate_store() -> Rc<dyn LenientPostmortemEstimateStore> {
    Rc::new(MemoryLenientPostmortemEstimateStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientPostmortemEstimateStore {
    pub inner: Rc<dyn LenientPostmortemEstimateStore>,
    pub cache: RefCell<cache::TtlCache<LenientPostmortemEstimate>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientPostmortemEstimateStore for CachedLenientPostmortemEstimateStore {
    fn find(&self, code: &str) -> Option<LenientPostmortemEstimate> {
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

    fn save(&self, item: &LenientPostmortemEstimate) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientPostmortemEstimate> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_postmortem_estimate_store(
    inner: Rc<dyn LenientPostmortemEstimateStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientPostmortemEstimateStore> {
    Rc::new(CachedLenientPostmortemEstimateStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientPostmortemEstimate>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient postmortem estimates. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPostmortemEstimateQuery {
    pub namespace: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientPostmortemEstimateQuery {
    pub fn matches(&self, item: &LenientPostmortemEstimate) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientPostmortemEstimate]) -> Vec<LenientPostmortemEstimate> {
        let matched = collections::filter(items, |candidate: &LenientPostmortemEstimate| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_postmortem_estimates(
    store: Rc<dyn LenientPostmortemEstimateStore>,
    query: &LenientPostmortemEstimateQuery,
) -> Vec<LenientPostmortemEstimate> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientPostmortemEstimateStore> = new_memory_lenient_postmortem_estimate_store();
        let item = new_lenient_postmortem_estimate("code-907", "title-448");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
