use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient evidence notices by code.
pub trait LenientEvidenceNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientEvidenceNotice>;
    fn save(&self, item: &LenientEvidenceNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientEvidenceNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientEvidenceNoticeStore {
    pub items: RefCell<BTreeMap<String, LenientEvidenceNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientEvidenceNoticeStore for MemoryLenientEvidenceNoticeStore {
    fn find(&self, code: &str) -> Option<LenientEvidenceNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientEvidenceNotice) {
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

    fn list(&self) -> Vec<LenientEvidenceNotice> {
        let mut out: Vec<LenientEvidenceNotice> = Vec::new();
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

pub fn new_memory_lenient_evidence_notice_store() -> Rc<dyn LenientEvidenceNoticeStore> {
    Rc::new(MemoryLenientEvidenceNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientEvidenceNoticeStore {
    pub inner: Rc<dyn LenientEvidenceNoticeStore>,
    pub cache: RefCell<cache::TtlCache<LenientEvidenceNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientEvidenceNoticeStore for CachedLenientEvidenceNoticeStore {
    fn find(&self, code: &str) -> Option<LenientEvidenceNotice> {
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

    fn save(&self, item: &LenientEvidenceNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientEvidenceNotice> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_evidence_notice_store(
    inner: Rc<dyn LenientEvidenceNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientEvidenceNoticeStore> {
    Rc::new(CachedLenientEvidenceNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientEvidenceNotice>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient evidence notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEvidenceNoticeQuery {
    pub title: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientEvidenceNoticeQuery {
    pub fn matches(&self, item: &LenientEvidenceNotice) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[LenientEvidenceNotice]) -> Vec<LenientEvidenceNotice> {
        let matched = collections::filter(items, |candidate: &LenientEvidenceNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_evidence_notices(
    store: Rc<dyn LenientEvidenceNoticeStore>,
    query: &LenientEvidenceNoticeQuery,
) -> Vec<LenientEvidenceNotice> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientEvidenceNoticeStore> = new_memory_lenient_evidence_notice_store();
        let item = new_lenient_evidence_notice("code-345", "title-926");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
