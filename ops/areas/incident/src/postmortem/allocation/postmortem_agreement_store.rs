use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps postmortem agreements by code.
pub trait PostmortemAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PostmortemAgreement>;
    fn save(&self, item: &PostmortemAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PostmortemAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPostmortemAgreementStore {
    pub items: RefCell<BTreeMap<String, PostmortemAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl PostmortemAgreementStore for MemoryPostmortemAgreementStore {
    fn find(&self, code: &str) -> Option<PostmortemAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PostmortemAgreement) {
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

    fn list(&self) -> Vec<PostmortemAgreement> {
        let mut out: Vec<PostmortemAgreement> = Vec::new();
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

pub fn new_memory_postmortem_agreement_store() -> Rc<dyn PostmortemAgreementStore> {
    Rc::new(MemoryPostmortemAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPostmortemAgreementStore {
    pub inner: Rc<dyn PostmortemAgreementStore>,
    pub cache: RefCell<cache::TtlCache<PostmortemAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PostmortemAgreementStore for CachedPostmortemAgreementStore {
    fn find(&self, code: &str) -> Option<PostmortemAgreement> {
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

    fn save(&self, item: &PostmortemAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PostmortemAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_postmortem_agreement_store(
    inner: Rc<dyn PostmortemAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PostmortemAgreementStore> {
    Rc::new(CachedPostmortemAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PostmortemAgreement>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects postmortem agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemAgreementQuery {
    pub unit: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PostmortemAgreementQuery {
    pub fn matches(&self, item: &PostmortemAgreement) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PostmortemAgreement]) -> Vec<PostmortemAgreement> {
        let matched = collections::filter(items, |candidate: &PostmortemAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_postmortem_agreements(
    store: Rc<dyn PostmortemAgreementStore>,
    query: &PostmortemAgreementQuery,
) -> Vec<PostmortemAgreement> {
    query.apply(&store.list())
}
