use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps evidence requests by code.
pub trait EvidenceRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EvidenceRequest>;
    fn save(&self, item: &EvidenceRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EvidenceRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEvidenceRequestStore {
    pub items: RefCell<BTreeMap<String, EvidenceRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl EvidenceRequestStore for MemoryEvidenceRequestStore {
    fn find(&self, code: &str) -> Option<EvidenceRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EvidenceRequest) {
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

    fn list(&self) -> Vec<EvidenceRequest> {
        let mut out: Vec<EvidenceRequest> = Vec::new();
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

pub fn new_memory_evidence_request_store() -> Rc<dyn EvidenceRequestStore> {
    Rc::new(MemoryEvidenceRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEvidenceRequestStore {
    pub inner: Rc<dyn EvidenceRequestStore>,
    pub cache: RefCell<cache::TtlCache<EvidenceRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EvidenceRequestStore for CachedEvidenceRequestStore {
    fn find(&self, code: &str) -> Option<EvidenceRequest> {
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

    fn save(&self, item: &EvidenceRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EvidenceRequest> {
        self.inner.list()
    }
}

pub fn new_cached_evidence_request_store(
    inner: Rc<dyn EvidenceRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EvidenceRequestStore> {
    Rc::new(CachedEvidenceRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EvidenceRequest>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects evidence requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRequestQuery {
    pub label: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EvidenceRequestQuery {
    pub fn matches(&self, item: &EvidenceRequest) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[EvidenceRequest]) -> Vec<EvidenceRequest> {
        let matched = collections::filter(items, |candidate: &EvidenceRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_evidence_requests(
    store: Rc<dyn EvidenceRequestStore>,
    query: &EvidenceRequestQuery,
) -> Vec<EvidenceRequest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn EvidenceRequestStore> = new_memory_evidence_request_store();
        let item = new_evidence_request("code-869", "label-855");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
