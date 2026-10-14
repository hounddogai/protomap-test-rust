use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient budget burn claims by code.
pub trait LenientBudgetBurnClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientBudgetBurnClaim>;
    fn save(&self, item: &LenientBudgetBurnClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientBudgetBurnClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientBudgetBurnClaimStore {
    pub items: RefCell<BTreeMap<String, LenientBudgetBurnClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientBudgetBurnClaimStore for MemoryLenientBudgetBurnClaimStore {
    fn find(&self, code: &str) -> Option<LenientBudgetBurnClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientBudgetBurnClaim) {
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

    fn list(&self) -> Vec<LenientBudgetBurnClaim> {
        let mut out: Vec<LenientBudgetBurnClaim> = Vec::new();
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

pub fn new_memory_lenient_budget_burn_claim_store() -> Rc<dyn LenientBudgetBurnClaimStore> {
    Rc::new(MemoryLenientBudgetBurnClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientBudgetBurnClaimStore {
    pub inner: Rc<dyn LenientBudgetBurnClaimStore>,
    pub cache: RefCell<cache::TtlCache<LenientBudgetBurnClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientBudgetBurnClaimStore for CachedLenientBudgetBurnClaimStore {
    fn find(&self, code: &str) -> Option<LenientBudgetBurnClaim> {
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

    fn save(&self, item: &LenientBudgetBurnClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientBudgetBurnClaim> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_budget_burn_claim_store(
    inner: Rc<dyn LenientBudgetBurnClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientBudgetBurnClaimStore> {
    Rc::new(CachedLenientBudgetBurnClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientBudgetBurnClaim>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient budget burn claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientBudgetBurnClaimQuery {
    pub namespace: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientBudgetBurnClaimQuery {
    pub fn matches(&self, item: &LenientBudgetBurnClaim) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientBudgetBurnClaim]) -> Vec<LenientBudgetBurnClaim> {
        let matched = collections::filter(items, |candidate: &LenientBudgetBurnClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_budget_burn_claims(
    store: Rc<dyn LenientBudgetBurnClaimStore>,
    query: &LenientBudgetBurnClaimQuery,
) -> Vec<LenientBudgetBurnClaim> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientBudgetBurnClaimStore> = new_memory_lenient_budget_burn_claim_store();
        let item = new_lenient_budget_burn_claim("code-525", "label-459");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
