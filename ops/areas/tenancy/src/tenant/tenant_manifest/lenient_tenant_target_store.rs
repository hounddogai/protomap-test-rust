use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient tenant targets by code.
pub trait LenientTenantTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientTenantTarget>;
    fn save(&self, item: &LenientTenantTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientTenantTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientTenantTargetStore {
    pub items: RefCell<BTreeMap<String, LenientTenantTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientTenantTargetStore for MemoryLenientTenantTargetStore {
    fn find(&self, code: &str) -> Option<LenientTenantTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientTenantTarget) {
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

    fn list(&self) -> Vec<LenientTenantTarget> {
        let mut out: Vec<LenientTenantTarget> = Vec::new();
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

pub fn new_memory_lenient_tenant_target_store() -> Rc<dyn LenientTenantTargetStore> {
    Rc::new(MemoryLenientTenantTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientTenantTargetStore {
    pub inner: Rc<dyn LenientTenantTargetStore>,
    pub cache: RefCell<cache::TtlCache<LenientTenantTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientTenantTargetStore for CachedLenientTenantTargetStore {
    fn find(&self, code: &str) -> Option<LenientTenantTarget> {
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

    fn save(&self, item: &LenientTenantTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientTenantTarget> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_tenant_target_store(
    inner: Rc<dyn LenientTenantTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientTenantTargetStore> {
    Rc::new(CachedLenientTenantTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientTenantTarget>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient tenant targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantTargetQuery {
    pub segment: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientTenantTargetQuery {
    pub fn matches(&self, item: &LenientTenantTarget) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientTenantTarget]) -> Vec<LenientTenantTarget> {
        let matched = collections::filter(items, |candidate: &LenientTenantTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_tenant_targets(
    store: Rc<dyn LenientTenantTargetStore>,
    query: &LenientTenantTargetQuery,
) -> Vec<LenientTenantTarget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientTenantTargetStore> = new_memory_lenient_tenant_target_store();
        let item = new_lenient_tenant_target("code-320", "name-909");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
