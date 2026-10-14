use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps region pin agreements by code.
pub trait RegionPinAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionPinAgreement>;
    fn save(&self, item: &RegionPinAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionPinAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionPinAgreementStore {
    pub items: RefCell<BTreeMap<String, RegionPinAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionPinAgreementStore for MemoryRegionPinAgreementStore {
    fn find(&self, code: &str) -> Option<RegionPinAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionPinAgreement) {
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

    fn list(&self) -> Vec<RegionPinAgreement> {
        let mut out: Vec<RegionPinAgreement> = Vec::new();
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

pub fn new_memory_region_pin_agreement_store() -> Rc<dyn RegionPinAgreementStore> {
    Rc::new(MemoryRegionPinAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionPinAgreementStore {
    pub inner: Rc<dyn RegionPinAgreementStore>,
    pub cache: RefCell<cache::TtlCache<RegionPinAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionPinAgreementStore for CachedRegionPinAgreementStore {
    fn find(&self, code: &str) -> Option<RegionPinAgreement> {
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

    fn save(&self, item: &RegionPinAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionPinAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_region_pin_agreement_store(
    inner: Rc<dyn RegionPinAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionPinAgreementStore> {
    Rc::new(CachedRegionPinAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionPinAgreement>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects region pin agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAgreementQuery {
    pub region: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionPinAgreementQuery {
    pub fn matches(&self, item: &RegionPinAgreement) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionPinAgreement]) -> Vec<RegionPinAgreement> {
        let matched = collections::filter(items, |candidate: &RegionPinAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_region_pin_agreements(
    store: Rc<dyn RegionPinAgreementStore>,
    query: &RegionPinAgreementQuery,
) -> Vec<RegionPinAgreement> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionPinAgreementStore> = new_memory_region_pin_agreement_store();
        let item = new_region_pin_agreement("code-117", "title-278");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
