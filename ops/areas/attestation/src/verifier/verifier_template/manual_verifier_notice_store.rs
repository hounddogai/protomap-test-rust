use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual verifier notices by code.
pub trait ManualVerifierNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualVerifierNotice>;
    fn save(&self, item: &ManualVerifierNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualVerifierNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualVerifierNoticeStore {
    pub items: RefCell<BTreeMap<String, ManualVerifierNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualVerifierNoticeStore for MemoryManualVerifierNoticeStore {
    fn find(&self, code: &str) -> Option<ManualVerifierNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualVerifierNotice) {
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

    fn list(&self) -> Vec<ManualVerifierNotice> {
        let mut out: Vec<ManualVerifierNotice> = Vec::new();
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

pub fn new_memory_manual_verifier_notice_store() -> Rc<dyn ManualVerifierNoticeStore> {
    Rc::new(MemoryManualVerifierNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualVerifierNoticeStore {
    pub inner: Rc<dyn ManualVerifierNoticeStore>,
    pub cache: RefCell<cache::TtlCache<ManualVerifierNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualVerifierNoticeStore for CachedManualVerifierNoticeStore {
    fn find(&self, code: &str) -> Option<ManualVerifierNotice> {
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

    fn save(&self, item: &ManualVerifierNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualVerifierNotice> {
        self.inner.list()
    }
}

pub fn new_cached_manual_verifier_notice_store(
    inner: Rc<dyn ManualVerifierNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualVerifierNoticeStore> {
    Rc::new(CachedManualVerifierNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualVerifierNotice>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual verifier notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierNoticeQuery {
    pub segment: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualVerifierNoticeQuery {
    pub fn matches(&self, item: &ManualVerifierNotice) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualVerifierNotice]) -> Vec<ManualVerifierNotice> {
        let matched = collections::filter(items, |candidate: &ManualVerifierNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_verifier_notices(
    store: Rc<dyn ManualVerifierNoticeStore>,
    query: &ManualVerifierNoticeQuery,
) -> Vec<ManualVerifierNotice> {
    query.apply(&store.list())
}
