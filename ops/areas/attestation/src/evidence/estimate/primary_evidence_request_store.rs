use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary evidence requests by code.
pub trait PrimaryEvidenceRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryEvidenceRequest>;
    fn save(&self, item: &PrimaryEvidenceRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryEvidenceRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryEvidenceRequestStore {
    pub items: RefCell<BTreeMap<String, PrimaryEvidenceRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryEvidenceRequestStore for MemoryPrimaryEvidenceRequestStore {
    fn find(&self, code: &str) -> Option<PrimaryEvidenceRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryEvidenceRequest) {
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

    fn list(&self) -> Vec<PrimaryEvidenceRequest> {
        let mut out: Vec<PrimaryEvidenceRequest> = Vec::new();
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

pub fn new_memory_primary_evidence_request_store() -> Rc<dyn PrimaryEvidenceRequestStore> {
    Rc::new(MemoryPrimaryEvidenceRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryEvidenceRequestStore {
    pub inner: Rc<dyn PrimaryEvidenceRequestStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryEvidenceRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryEvidenceRequestStore for CachedPrimaryEvidenceRequestStore {
    fn find(&self, code: &str) -> Option<PrimaryEvidenceRequest> {
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

    fn save(&self, item: &PrimaryEvidenceRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryEvidenceRequest> {
        self.inner.list()
    }
}

pub fn new_cached_primary_evidence_request_store(
    inner: Rc<dyn PrimaryEvidenceRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryEvidenceRequestStore> {
    Rc::new(CachedPrimaryEvidenceRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryEvidenceRequest>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary evidence requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceRequestQuery {
    pub category: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryEvidenceRequestQuery {
    pub fn matches(&self, item: &PrimaryEvidenceRequest) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[PrimaryEvidenceRequest]) -> Vec<PrimaryEvidenceRequest> {
        let matched = collections::filter(items, |candidate: &PrimaryEvidenceRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_evidence_requests(
    store: Rc<dyn PrimaryEvidenceRequestStore>,
    query: &PrimaryEvidenceRequestQuery,
) -> Vec<PrimaryEvidenceRequest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryEvidenceRequestStore> = new_memory_primary_evidence_request_store();
        let item = new_primary_evidence_request("code-375", "name-724");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
