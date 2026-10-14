use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback suspension agreements by code.
pub trait FallbackSuspensionAgreementStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackSuspensionAgreement>;
    fn save(&self, item: &FallbackSuspensionAgreement);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackSuspensionAgreement>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackSuspensionAgreementStore {
    pub items: RefCell<BTreeMap<String, FallbackSuspensionAgreement>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackSuspensionAgreementStore for MemoryFallbackSuspensionAgreementStore {
    fn find(&self, code: &str) -> Option<FallbackSuspensionAgreement> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackSuspensionAgreement) {
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

    fn list(&self) -> Vec<FallbackSuspensionAgreement> {
        let mut out: Vec<FallbackSuspensionAgreement> = Vec::new();
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

pub fn new_memory_fallback_suspension_agreement_store() -> Rc<dyn FallbackSuspensionAgreementStore> {
    Rc::new(MemoryFallbackSuspensionAgreementStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackSuspensionAgreementStore {
    pub inner: Rc<dyn FallbackSuspensionAgreementStore>,
    pub cache: RefCell<cache::TtlCache<FallbackSuspensionAgreement>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackSuspensionAgreementStore for CachedFallbackSuspensionAgreementStore {
    fn find(&self, code: &str) -> Option<FallbackSuspensionAgreement> {
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

    fn save(&self, item: &FallbackSuspensionAgreement) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackSuspensionAgreement> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_suspension_agreement_store(
    inner: Rc<dyn FallbackSuspensionAgreementStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackSuspensionAgreementStore> {
    Rc::new(CachedFallbackSuspensionAgreementStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackSuspensionAgreement>(
            64,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback suspension agreements. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSuspensionAgreementQuery {
    pub name: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackSuspensionAgreementQuery {
    pub fn matches(&self, item: &FallbackSuspensionAgreement) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[FallbackSuspensionAgreement]) -> Vec<FallbackSuspensionAgreement> {
        let matched = collections::filter(items, |candidate: &FallbackSuspensionAgreement| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_suspension_agreements(
    store: Rc<dyn FallbackSuspensionAgreementStore>,
    query: &FallbackSuspensionAgreementQuery,
) -> Vec<FallbackSuspensionAgreement> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackSuspensionAgreementStore> = new_memory_fallback_suspension_agreement_store();
        let item = new_fallback_suspension_agreement("code-424", "name-484");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
