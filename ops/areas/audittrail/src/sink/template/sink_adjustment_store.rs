use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps sink adjustments by code.
pub trait SinkAdjustmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SinkAdjustment>;
    fn save(&self, item: &SinkAdjustment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SinkAdjustment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySinkAdjustmentStore {
    pub items: RefCell<BTreeMap<String, SinkAdjustment>>,
    pub order: RefCell<Vec<String>>,
}

impl SinkAdjustmentStore for MemorySinkAdjustmentStore {
    fn find(&self, code: &str) -> Option<SinkAdjustment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SinkAdjustment) {
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

    fn list(&self) -> Vec<SinkAdjustment> {
        let mut out: Vec<SinkAdjustment> = Vec::new();
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

pub fn new_memory_sink_adjustment_store() -> Rc<dyn SinkAdjustmentStore> {
    Rc::new(MemorySinkAdjustmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSinkAdjustmentStore {
    pub inner: Rc<dyn SinkAdjustmentStore>,
    pub cache: RefCell<cache::TtlCache<SinkAdjustment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SinkAdjustmentStore for CachedSinkAdjustmentStore {
    fn find(&self, code: &str) -> Option<SinkAdjustment> {
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

    fn save(&self, item: &SinkAdjustment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SinkAdjustment> {
        self.inner.list()
    }
}

pub fn new_cached_sink_adjustment_store(
    inner: Rc<dyn SinkAdjustmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SinkAdjustmentStore> {
    Rc::new(CachedSinkAdjustmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SinkAdjustment>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects sink adjustments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkAdjustmentQuery {
    pub segment: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SinkAdjustmentQuery {
    pub fn matches(&self, item: &SinkAdjustment) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SinkAdjustment]) -> Vec<SinkAdjustment> {
        let matched = collections::filter(items, |candidate: &SinkAdjustment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_sink_adjustments(store: Rc<dyn SinkAdjustmentStore>, query: &SinkAdjustmentQuery) -> Vec<SinkAdjustment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SinkAdjustmentStore> = new_memory_sink_adjustment_store();
        let item = new_sink_adjustment("code-399", "title-173");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
