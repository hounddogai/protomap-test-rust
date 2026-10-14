use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps plan agreements by code.
pub trait PlanAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PlanAgreement>;
    fn save(&self, item: &PlanAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PlanAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPlanAgreementStore {
    pub items: RefCell<BTreeMap<String, PlanAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl PlanAgreementStore for MemoryPlanAgreementStore {
    fn find(&self, code: &str) -> Option<PlanAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PlanAgreement) {
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

    fn list(&self) -> Vec<PlanAgreement> {
        let mut out: Vec<PlanAgreement> = Vec::new();
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

pub fn new_memory_plan_agreement_store() -> Rc<dyn PlanAgreementStore> {
    Rc::new(MemoryPlanAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPlanAgreementStore {
    pub inner: Rc<dyn PlanAgreementStore>,
    pub cache: RefCell<cache::TtlCache<PlanAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PlanAgreementStore for CachedPlanAgreementStore {
    fn find(&self, code: &str) -> Option<PlanAgreement> {
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

    fn save(&self, item: &PlanAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PlanAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_plan_agreement_store(
    inner: Rc<dyn PlanAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PlanAgreementStore> {
    Rc::new(CachedPlanAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PlanAgreement>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects plan agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanAgreementQuery {
    pub segment: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PlanAgreementQuery {
    pub fn matches(&self, item: &PlanAgreement) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PlanAgreement]) -> Vec<PlanAgreement> {
        let matched = collections::filter(items, |candidate: &PlanAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_plan_agreements(store: Rc<dyn PlanAgreementStore>, query: &PlanAgreementQuery) -> Vec<PlanAgreement> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PlanAgreementStore> = new_memory_plan_agreement_store();
        let item = new_plan_agreement("code-630", "title-231");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
