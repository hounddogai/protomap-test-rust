use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy rollback policies by code.
pub trait LegacyRollbackPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyRollbackPolicy>;
    fn save(&self, item: &LegacyRollbackPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyRollbackPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyRollbackPolicyStore {
    pub items: RefCell<BTreeMap<String, LegacyRollbackPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyRollbackPolicyStore for MemoryLegacyRollbackPolicyStore {
    fn find(&self, code: &str) -> Option<LegacyRollbackPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyRollbackPolicy) {
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

    fn list(&self) -> Vec<LegacyRollbackPolicy> {
        let mut out: Vec<LegacyRollbackPolicy> = Vec::new();
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

pub fn new_memory_legacy_rollback_policy_store() -> Rc<dyn LegacyRollbackPolicyStore> {
    Rc::new(MemoryLegacyRollbackPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyRollbackPolicyStore {
    pub inner: Rc<dyn LegacyRollbackPolicyStore>,
    pub cache: RefCell<cache::TtlCache<LegacyRollbackPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyRollbackPolicyStore for CachedLegacyRollbackPolicyStore {
    fn find(&self, code: &str) -> Option<LegacyRollbackPolicy> {
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

    fn save(&self, item: &LegacyRollbackPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyRollbackPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_rollback_policy_store(
    inner: Rc<dyn LegacyRollbackPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyRollbackPolicyStore> {
    Rc::new(CachedLegacyRollbackPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyRollbackPolicy>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy rollback policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackPolicyQuery {
    pub locale: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyRollbackPolicyQuery {
    pub fn matches(&self, item: &LegacyRollbackPolicy) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyRollbackPolicy]) -> Vec<LegacyRollbackPolicy> {
        let matched = collections::filter(items, |candidate: &LegacyRollbackPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_rollback_policies(
    store: Rc<dyn LegacyRollbackPolicyStore>,
    query: &LegacyRollbackPolicyQuery,
) -> Vec<LegacyRollbackPolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyRollbackPolicyStore> = new_memory_legacy_rollback_policy_store();
        let item = new_legacy_rollback_policy("code-725", "title-944");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
