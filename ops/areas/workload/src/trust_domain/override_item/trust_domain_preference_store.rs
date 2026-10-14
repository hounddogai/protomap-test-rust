use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps trust domain preferences by code.
pub trait TrustDomainPreferenceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TrustDomainPreference>;
    fn save(&self, item: &TrustDomainPreference);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TrustDomainPreference>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTrustDomainPreferenceStore {
    pub items: RefCell<BTreeMap<String, TrustDomainPreference>>,
    pub order: RefCell<Vec<String>>,
}

impl TrustDomainPreferenceStore for MemoryTrustDomainPreferenceStore {
    fn find(&self, code: &str) -> Option<TrustDomainPreference> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TrustDomainPreference) {
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

    fn list(&self) -> Vec<TrustDomainPreference> {
        let mut out: Vec<TrustDomainPreference> = Vec::new();
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

pub fn new_memory_trust_domain_preference_store() -> Rc<dyn TrustDomainPreferenceStore> {
    Rc::new(MemoryTrustDomainPreferenceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTrustDomainPreferenceStore {
    pub inner: Rc<dyn TrustDomainPreferenceStore>,
    pub cache: RefCell<cache::TtlCache<TrustDomainPreference>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TrustDomainPreferenceStore for CachedTrustDomainPreferenceStore {
    fn find(&self, code: &str) -> Option<TrustDomainPreference> {
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

    fn save(&self, item: &TrustDomainPreference) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TrustDomainPreference> {
        self.inner.list()
    }
}

pub fn new_cached_trust_domain_preference_store(
    inner: Rc<dyn TrustDomainPreferenceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TrustDomainPreferenceStore> {
    Rc::new(CachedTrustDomainPreferenceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TrustDomainPreference>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects trust domain preferences. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainPreferenceQuery {
    pub cluster: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TrustDomainPreferenceQuery {
    pub fn matches(&self, item: &TrustDomainPreference) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TrustDomainPreference]) -> Vec<TrustDomainPreference> {
        let matched = collections::filter(items, |candidate: &TrustDomainPreference| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_trust_domain_preferences(
    store: Rc<dyn TrustDomainPreferenceStore>,
    query: &TrustDomainPreferenceQuery,
) -> Vec<TrustDomainPreference> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TrustDomainPreferenceStore> = new_memory_trust_domain_preference_store();
        let item = new_trust_domain_preference("code-297", "name-331");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
