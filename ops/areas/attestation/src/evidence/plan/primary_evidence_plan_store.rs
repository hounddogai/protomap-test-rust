use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary evidence plans by code.
pub trait PrimaryEvidencePlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryEvidencePlan>;
    fn save(&self, item: &PrimaryEvidencePlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryEvidencePlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryEvidencePlanStore {
    pub items: RefCell<BTreeMap<String, PrimaryEvidencePlan>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryEvidencePlanStore for MemoryPrimaryEvidencePlanStore {
    fn find(&self, code: &str) -> Option<PrimaryEvidencePlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryEvidencePlan) {
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

    fn list(&self) -> Vec<PrimaryEvidencePlan> {
        let mut out: Vec<PrimaryEvidencePlan> = Vec::new();
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

pub fn new_memory_primary_evidence_plan_store() -> Rc<dyn PrimaryEvidencePlanStore> {
    Rc::new(MemoryPrimaryEvidencePlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryEvidencePlanStore {
    pub inner: Rc<dyn PrimaryEvidencePlanStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryEvidencePlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryEvidencePlanStore for CachedPrimaryEvidencePlanStore {
    fn find(&self, code: &str) -> Option<PrimaryEvidencePlan> {
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

    fn save(&self, item: &PrimaryEvidencePlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryEvidencePlan> {
        self.inner.list()
    }
}

pub fn new_cached_primary_evidence_plan_store(
    inner: Rc<dyn PrimaryEvidencePlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryEvidencePlanStore> {
    Rc::new(CachedPrimaryEvidencePlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryEvidencePlan>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary evidence plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidencePlanQuery {
    pub channel: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryEvidencePlanQuery {
    pub fn matches(&self, item: &PrimaryEvidencePlan) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
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

    pub fn apply(&self, items: &[PrimaryEvidencePlan]) -> Vec<PrimaryEvidencePlan> {
        let matched = collections::filter(items, |candidate: &PrimaryEvidencePlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_evidence_plans(
    store: Rc<dyn PrimaryEvidencePlanStore>,
    query: &PrimaryEvidencePlanQuery,
) -> Vec<PrimaryEvidencePlan> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryEvidencePlanStore> = new_memory_primary_evidence_plan_store();
        let item = new_primary_evidence_plan("code-366", "name-183");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
