use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional probe claims by code.
pub trait RegionalProbeClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalProbeClaim>;
    fn save(&self, item: &RegionalProbeClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalProbeClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalProbeClaimStore {
    pub items: RefCell<BTreeMap<String, RegionalProbeClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalProbeClaimStore for MemoryRegionalProbeClaimStore {
    fn find(&self, code: &str) -> Option<RegionalProbeClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalProbeClaim) {
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

    fn list(&self) -> Vec<RegionalProbeClaim> {
        let mut out: Vec<RegionalProbeClaim> = Vec::new();
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

pub fn new_memory_regional_probe_claim_store() -> Rc<dyn RegionalProbeClaimStore> {
    Rc::new(MemoryRegionalProbeClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalProbeClaimStore {
    pub inner: Rc<dyn RegionalProbeClaimStore>,
    pub cache: RefCell<cache::TtlCache<RegionalProbeClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalProbeClaimStore for CachedRegionalProbeClaimStore {
    fn find(&self, code: &str) -> Option<RegionalProbeClaim> {
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

    fn save(&self, item: &RegionalProbeClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalProbeClaim> {
        self.inner.list()
    }
}

pub fn new_cached_regional_probe_claim_store(
    inner: Rc<dyn RegionalProbeClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalProbeClaimStore> {
    Rc::new(CachedRegionalProbeClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalProbeClaim>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional probe claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeClaimQuery {
    pub label: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalProbeClaimQuery {
    pub fn matches(&self, item: &RegionalProbeClaim) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.expires_at < self.min_expires_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalProbeClaim]) -> Vec<RegionalProbeClaim> {
        let matched = collections::filter(items, |candidate: &RegionalProbeClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_probe_claims(
    store: Rc<dyn RegionalProbeClaimStore>,
    query: &RegionalProbeClaimQuery,
) -> Vec<RegionalProbeClaim> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalProbeClaimStore> = new_memory_regional_probe_claim_store();
        let item = new_regional_probe_claim("code-497", "label-46");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
