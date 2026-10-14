use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled segment baselines by code.
pub trait ScheduledSegmentBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledSegmentBaseline>;
    fn save(&self, item: &ScheduledSegmentBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledSegmentBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledSegmentBaselineStore {
    pub items: RefCell<BTreeMap<String, ScheduledSegmentBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledSegmentBaselineStore for MemoryScheduledSegmentBaselineStore {
    fn find(&self, code: &str) -> Option<ScheduledSegmentBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledSegmentBaseline) {
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

    fn list(&self) -> Vec<ScheduledSegmentBaseline> {
        let mut out: Vec<ScheduledSegmentBaseline> = Vec::new();
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

pub fn new_memory_scheduled_segment_baseline_store() -> Rc<dyn ScheduledSegmentBaselineStore> {
    Rc::new(MemoryScheduledSegmentBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledSegmentBaselineStore {
    pub inner: Rc<dyn ScheduledSegmentBaselineStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledSegmentBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledSegmentBaselineStore for CachedScheduledSegmentBaselineStore {
    fn find(&self, code: &str) -> Option<ScheduledSegmentBaseline> {
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

    fn save(&self, item: &ScheduledSegmentBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledSegmentBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_segment_baseline_store(
    inner: Rc<dyn ScheduledSegmentBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledSegmentBaselineStore> {
    Rc::new(CachedScheduledSegmentBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledSegmentBaseline>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled segment baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentBaselineQuery {
    pub name: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledSegmentBaselineQuery {
    pub fn matches(&self, item: &ScheduledSegmentBaseline) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledSegmentBaseline]) -> Vec<ScheduledSegmentBaseline> {
        let matched = collections::filter(items, |candidate: &ScheduledSegmentBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_segment_baselines(
    store: Rc<dyn ScheduledSegmentBaselineStore>,
    query: &ScheduledSegmentBaselineQuery,
) -> Vec<ScheduledSegmentBaseline> {
    query.apply(&store.list())
}
