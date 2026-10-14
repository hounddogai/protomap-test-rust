use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy measurement notices by code.
pub trait LegacyMeasurementNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyMeasurementNotice>;
    fn save(&self, item: &LegacyMeasurementNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyMeasurementNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyMeasurementNoticeStore {
    pub items: RefCell<BTreeMap<String, LegacyMeasurementNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyMeasurementNoticeStore for MemoryLegacyMeasurementNoticeStore {
    fn find(&self, code: &str) -> Option<LegacyMeasurementNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyMeasurementNotice) {
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

    fn list(&self) -> Vec<LegacyMeasurementNotice> {
        let mut out: Vec<LegacyMeasurementNotice> = Vec::new();
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

pub fn new_memory_legacy_measurement_notice_store() -> Rc<dyn LegacyMeasurementNoticeStore> {
    Rc::new(MemoryLegacyMeasurementNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyMeasurementNoticeStore {
    pub inner: Rc<dyn LegacyMeasurementNoticeStore>,
    pub cache: RefCell<cache::TtlCache<LegacyMeasurementNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyMeasurementNoticeStore for CachedLegacyMeasurementNoticeStore {
    fn find(&self, code: &str) -> Option<LegacyMeasurementNotice> {
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

    fn save(&self, item: &LegacyMeasurementNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyMeasurementNotice> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_measurement_notice_store(
    inner: Rc<dyn LegacyMeasurementNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyMeasurementNoticeStore> {
    Rc::new(CachedLegacyMeasurementNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyMeasurementNotice>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy measurement notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementNoticeQuery {
    pub source: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyMeasurementNoticeQuery {
    pub fn matches(&self, item: &LegacyMeasurementNotice) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
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

    pub fn apply(&self, items: &[LegacyMeasurementNotice]) -> Vec<LegacyMeasurementNotice> {
        let matched = collections::filter(items, |candidate: &LegacyMeasurementNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_measurement_notices(
    store: Rc<dyn LegacyMeasurementNoticeStore>,
    query: &LegacyMeasurementNoticeQuery,
) -> Vec<LegacyMeasurementNotice> {
    query.apply(&store.list())
}
