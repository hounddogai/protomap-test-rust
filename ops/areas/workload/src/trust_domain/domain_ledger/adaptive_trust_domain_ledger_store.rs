use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive trust domain ledgers by code.
pub trait AdaptiveTrustDomainLedgerStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveTrustDomainLedger>;
    fn save(&self, item: &AdaptiveTrustDomainLedger);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveTrustDomainLedger>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveTrustDomainLedgerStore {
    pub items: RefCell<BTreeMap<String, AdaptiveTrustDomainLedger>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveTrustDomainLedgerStore for MemoryAdaptiveTrustDomainLedgerStore {
    fn find(&self, code: &str) -> Option<AdaptiveTrustDomainLedger> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveTrustDomainLedger) {
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

    fn list(&self) -> Vec<AdaptiveTrustDomainLedger> {
        let mut out: Vec<AdaptiveTrustDomainLedger> = Vec::new();
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

pub fn new_memory_adaptive_trust_domain_ledger_store() -> Rc<dyn AdaptiveTrustDomainLedgerStore> {
    Rc::new(MemoryAdaptiveTrustDomainLedgerStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveTrustDomainLedgerStore {
    pub inner: Rc<dyn AdaptiveTrustDomainLedgerStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveTrustDomainLedger>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveTrustDomainLedgerStore for CachedAdaptiveTrustDomainLedgerStore {
    fn find(&self, code: &str) -> Option<AdaptiveTrustDomainLedger> {
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

    fn save(&self, item: &AdaptiveTrustDomainLedger) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveTrustDomainLedger> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_trust_domain_ledger_store(
    inner: Rc<dyn AdaptiveTrustDomainLedgerStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveTrustDomainLedgerStore> {
    Rc::new(CachedAdaptiveTrustDomainLedgerStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveTrustDomainLedger>(
            64,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive trust domain ledgers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainLedgerQuery {
    pub namespace: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveTrustDomainLedgerQuery {
    pub fn matches(&self, item: &AdaptiveTrustDomainLedger) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveTrustDomainLedger]) -> Vec<AdaptiveTrustDomainLedger> {
        let matched = collections::filter(items, |candidate: &AdaptiveTrustDomainLedger| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_trust_domain_ledgers(
    store: Rc<dyn AdaptiveTrustDomainLedgerStore>,
    query: &AdaptiveTrustDomainLedgerQuery,
) -> Vec<AdaptiveTrustDomainLedger> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AdaptiveTrustDomainLedgerStore> = new_memory_adaptive_trust_domain_ledger_store();
        let item = new_adaptive_trust_domain_ledger("code-877", "title-508");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
