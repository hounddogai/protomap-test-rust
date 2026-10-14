use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps plan segments by code.
pub trait PlanSegmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PlanSegment>;
    fn save(&self, item: &PlanSegment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PlanSegment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPlanSegmentStore {
    pub items: RefCell<BTreeMap<String, PlanSegment>>,
    pub order: RefCell<Vec<String>>,
}

impl PlanSegmentStore for MemoryPlanSegmentStore {
    fn find(&self, code: &str) -> Option<PlanSegment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PlanSegment) {
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

    fn list(&self) -> Vec<PlanSegment> {
        let mut out: Vec<PlanSegment> = Vec::new();
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

pub fn new_memory_plan_segment_store() -> Rc<dyn PlanSegmentStore> {
    Rc::new(MemoryPlanSegmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPlanSegmentStore {
    pub inner: Rc<dyn PlanSegmentStore>,
    pub cache: RefCell<cache::TtlCache<PlanSegment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PlanSegmentStore for CachedPlanSegmentStore {
    fn find(&self, code: &str) -> Option<PlanSegment> {
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

    fn save(&self, item: &PlanSegment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PlanSegment> {
        self.inner.list()
    }
}

pub fn new_cached_plan_segment_store(
    inner: Rc<dyn PlanSegmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PlanSegmentStore> {
    Rc::new(CachedPlanSegmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PlanSegment>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects plan segments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanSegmentQuery {
    pub name: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PlanSegmentQuery {
    pub fn matches(&self, item: &PlanSegment) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[PlanSegment]) -> Vec<PlanSegment> {
        let matched = collections::filter(items, |candidate: &PlanSegment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_plan_segments(store: Rc<dyn PlanSegmentStore>, query: &PlanSegmentQuery) -> Vec<PlanSegment> {
    query.apply(&store.list())
}
