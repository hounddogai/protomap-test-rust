use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary evidence notices by code.
pub trait PrimaryEvidenceNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryEvidenceNotice>;
    fn save(&self, item: &PrimaryEvidenceNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryEvidenceNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryEvidenceNoticeStore {
    pub items: RefCell<BTreeMap<String, PrimaryEvidenceNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryEvidenceNoticeStore for MemoryPrimaryEvidenceNoticeStore {
    fn find(&self, code: &str) -> Option<PrimaryEvidenceNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryEvidenceNotice) {
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

    fn list(&self) -> Vec<PrimaryEvidenceNotice> {
        let mut out: Vec<PrimaryEvidenceNotice> = Vec::new();
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

pub fn new_memory_primary_evidence_notice_store() -> Rc<dyn PrimaryEvidenceNoticeStore> {
    Rc::new(MemoryPrimaryEvidenceNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryEvidenceNoticeStore {
    pub inner: Rc<dyn PrimaryEvidenceNoticeStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryEvidenceNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryEvidenceNoticeStore for CachedPrimaryEvidenceNoticeStore {
    fn find(&self, code: &str) -> Option<PrimaryEvidenceNotice> {
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

    fn save(&self, item: &PrimaryEvidenceNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryEvidenceNotice> {
        self.inner.list()
    }
}

pub fn new_cached_primary_evidence_notice_store(
    inner: Rc<dyn PrimaryEvidenceNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryEvidenceNoticeStore> {
    Rc::new(CachedPrimaryEvidenceNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryEvidenceNotice>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary evidence notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceNoticeQuery {
    pub scope: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryEvidenceNoticeQuery {
    pub fn matches(&self, item: &PrimaryEvidenceNotice) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
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

    pub fn apply(&self, items: &[PrimaryEvidenceNotice]) -> Vec<PrimaryEvidenceNotice> {
        let matched = collections::filter(items, |candidate: &PrimaryEvidenceNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_evidence_notices(
    store: Rc<dyn PrimaryEvidenceNoticeStore>,
    query: &PrimaryEvidenceNoticeQuery,
) -> Vec<PrimaryEvidenceNotice> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryEvidenceNoticeStore> = new_memory_primary_evidence_notice_store();
        let item = new_primary_evidence_notice("code-788", "name-757");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
