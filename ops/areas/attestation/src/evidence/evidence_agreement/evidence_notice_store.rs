use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps evidence notices by code.
pub trait EvidenceNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EvidenceNotice>;
    fn save(&self, item: &EvidenceNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EvidenceNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEvidenceNoticeStore {
    pub items: RefCell<BTreeMap<String, EvidenceNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl EvidenceNoticeStore for MemoryEvidenceNoticeStore {
    fn find(&self, code: &str) -> Option<EvidenceNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EvidenceNotice) {
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

    fn list(&self) -> Vec<EvidenceNotice> {
        let mut out: Vec<EvidenceNotice> = Vec::new();
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

pub fn new_memory_evidence_notice_store() -> Rc<dyn EvidenceNoticeStore> {
    Rc::new(MemoryEvidenceNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEvidenceNoticeStore {
    pub inner: Rc<dyn EvidenceNoticeStore>,
    pub cache: RefCell<cache::TtlCache<EvidenceNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EvidenceNoticeStore for CachedEvidenceNoticeStore {
    fn find(&self, code: &str) -> Option<EvidenceNotice> {
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

    fn save(&self, item: &EvidenceNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EvidenceNotice> {
        self.inner.list()
    }
}

pub fn new_cached_evidence_notice_store(
    inner: Rc<dyn EvidenceNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EvidenceNoticeStore> {
    Rc::new(CachedEvidenceNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EvidenceNotice>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects evidence notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceNoticeQuery {
    pub locale: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EvidenceNoticeQuery {
    pub fn matches(&self, item: &EvidenceNotice) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[EvidenceNotice]) -> Vec<EvidenceNotice> {
        let matched = collections::filter(items, |candidate: &EvidenceNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_evidence_notices(store: Rc<dyn EvidenceNoticeStore>, query: &EvidenceNoticeQuery) -> Vec<EvidenceNotice> {
    query.apply(&store.list())
}
