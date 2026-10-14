use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending region pin agreements by code.
pub trait PendingRegionPinAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingRegionPinAgreement>;
    fn save(&self, item: &PendingRegionPinAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingRegionPinAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingRegionPinAgreementStore {
    pub items: RefCell<BTreeMap<String, PendingRegionPinAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingRegionPinAgreementStore for MemoryPendingRegionPinAgreementStore {
    fn find(&self, code: &str) -> Option<PendingRegionPinAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingRegionPinAgreement) {
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

    fn list(&self) -> Vec<PendingRegionPinAgreement> {
        let mut out: Vec<PendingRegionPinAgreement> = Vec::new();
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

pub fn new_memory_pending_region_pin_agreement_store() -> Rc<dyn PendingRegionPinAgreementStore> {
    Rc::new(MemoryPendingRegionPinAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingRegionPinAgreementStore {
    pub inner: Rc<dyn PendingRegionPinAgreementStore>,
    pub cache: RefCell<cache::TtlCache<PendingRegionPinAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingRegionPinAgreementStore for CachedPendingRegionPinAgreementStore {
    fn find(&self, code: &str) -> Option<PendingRegionPinAgreement> {
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

    fn save(&self, item: &PendingRegionPinAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingRegionPinAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_pending_region_pin_agreement_store(
    inner: Rc<dyn PendingRegionPinAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingRegionPinAgreementStore> {
    Rc::new(CachedPendingRegionPinAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingRegionPinAgreement>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending region pin agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinAgreementQuery {
    pub category: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingRegionPinAgreementQuery {
    pub fn matches(&self, item: &PendingRegionPinAgreement) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[PendingRegionPinAgreement]) -> Vec<PendingRegionPinAgreement> {
        let matched = collections::filter(items, |candidate: &PendingRegionPinAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_region_pin_agreements(
    store: Rc<dyn PendingRegionPinAgreementStore>,
    query: &PendingRegionPinAgreementQuery,
) -> Vec<PendingRegionPinAgreement> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PendingRegionPinAgreementStore> = new_memory_pending_region_pin_agreement_store();
        let item = new_pending_region_pin_agreement("code-507", "title-228");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
