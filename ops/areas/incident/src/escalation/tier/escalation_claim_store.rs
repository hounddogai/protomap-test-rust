use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps escalation claims by code.
pub trait EscalationClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EscalationClaim>;
    fn save(&self, item: &EscalationClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EscalationClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEscalationClaimStore {
    pub items: RefCell<BTreeMap<String, EscalationClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl EscalationClaimStore for MemoryEscalationClaimStore {
    fn find(&self, code: &str) -> Option<EscalationClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EscalationClaim) {
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

    fn list(&self) -> Vec<EscalationClaim> {
        let mut out: Vec<EscalationClaim> = Vec::new();
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

pub fn new_memory_escalation_claim_store() -> Rc<dyn EscalationClaimStore> {
    Rc::new(MemoryEscalationClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEscalationClaimStore {
    pub inner: Rc<dyn EscalationClaimStore>,
    pub cache: RefCell<cache::TtlCache<EscalationClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EscalationClaimStore for CachedEscalationClaimStore {
    fn find(&self, code: &str) -> Option<EscalationClaim> {
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

    fn save(&self, item: &EscalationClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EscalationClaim> {
        self.inner.list()
    }
}

pub fn new_cached_escalation_claim_store(
    inner: Rc<dyn EscalationClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EscalationClaimStore> {
    Rc::new(CachedEscalationClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EscalationClaim>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects escalation claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationClaimQuery {
    pub scope: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EscalationClaimQuery {
    pub fn matches(&self, item: &EscalationClaim) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.attempts < self.min_attempts {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[EscalationClaim]) -> Vec<EscalationClaim> {
        let matched = collections::filter(items, |candidate: &EscalationClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_escalation_claims(
    store: Rc<dyn EscalationClaimStore>,
    query: &EscalationClaimQuery,
) -> Vec<EscalationClaim> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn EscalationClaimStore> = new_memory_escalation_claim_store();
        let item = new_escalation_claim("code-867", "title-165");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
