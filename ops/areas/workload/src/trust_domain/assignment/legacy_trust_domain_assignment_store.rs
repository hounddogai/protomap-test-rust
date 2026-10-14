use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy trust domain assignments by code.
pub trait LegacyTrustDomainAssignmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyTrustDomainAssignment>;
    fn save(&self, item: &LegacyTrustDomainAssignment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyTrustDomainAssignment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyTrustDomainAssignmentStore {
    pub items: RefCell<BTreeMap<String, LegacyTrustDomainAssignment>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyTrustDomainAssignmentStore for MemoryLegacyTrustDomainAssignmentStore {
    fn find(&self, code: &str) -> Option<LegacyTrustDomainAssignment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyTrustDomainAssignment) {
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

    fn list(&self) -> Vec<LegacyTrustDomainAssignment> {
        let mut out: Vec<LegacyTrustDomainAssignment> = Vec::new();
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

pub fn new_memory_legacy_trust_domain_assignment_store() -> Rc<dyn LegacyTrustDomainAssignmentStore> {
    Rc::new(MemoryLegacyTrustDomainAssignmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyTrustDomainAssignmentStore {
    pub inner: Rc<dyn LegacyTrustDomainAssignmentStore>,
    pub cache: RefCell<cache::TtlCache<LegacyTrustDomainAssignment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyTrustDomainAssignmentStore for CachedLegacyTrustDomainAssignmentStore {
    fn find(&self, code: &str) -> Option<LegacyTrustDomainAssignment> {
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

    fn save(&self, item: &LegacyTrustDomainAssignment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyTrustDomainAssignment> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_trust_domain_assignment_store(
    inner: Rc<dyn LegacyTrustDomainAssignmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyTrustDomainAssignmentStore> {
    Rc::new(CachedLegacyTrustDomainAssignmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyTrustDomainAssignment>(
            64,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy trust domain assignments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainAssignmentQuery {
    pub source: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyTrustDomainAssignmentQuery {
    pub fn matches(&self, item: &LegacyTrustDomainAssignment) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyTrustDomainAssignment]) -> Vec<LegacyTrustDomainAssignment> {
        let matched = collections::filter(items, |candidate: &LegacyTrustDomainAssignment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_trust_domain_assignments(
    store: Rc<dyn LegacyTrustDomainAssignmentStore>,
    query: &LegacyTrustDomainAssignmentQuery,
) -> Vec<LegacyTrustDomainAssignment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyTrustDomainAssignmentStore> = new_memory_legacy_trust_domain_assignment_store();
        let item = new_legacy_trust_domain_assignment("code-375", "title-910");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
