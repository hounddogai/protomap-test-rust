use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered credential plans by code.
pub trait TieredCredentialPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredCredentialPlan>;
    fn save(&self, item: &TieredCredentialPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredCredentialPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredCredentialPlanStore {
    pub items: RefCell<BTreeMap<String, TieredCredentialPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredCredentialPlanStore for MemoryTieredCredentialPlanStore {
    fn find(&self, code: &str) -> Option<TieredCredentialPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredCredentialPlan) {
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

    fn list(&self) -> Vec<TieredCredentialPlan> {
        let mut out: Vec<TieredCredentialPlan> = Vec::new();
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

pub fn new_memory_tiered_credential_plan_store() -> Rc<dyn TieredCredentialPlanStore> {
    Rc::new(MemoryTieredCredentialPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredCredentialPlanStore {
    pub inner: Rc<dyn TieredCredentialPlanStore>,
    pub cache: RefCell<cache::TtlCache<TieredCredentialPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredCredentialPlanStore for CachedTieredCredentialPlanStore {
    fn find(&self, code: &str) -> Option<TieredCredentialPlan> {
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

    fn save(&self, item: &TieredCredentialPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredCredentialPlan> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_credential_plan_store(
    inner: Rc<dyn TieredCredentialPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredCredentialPlanStore> {
    Rc::new(CachedTieredCredentialPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredCredentialPlan>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered credential plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialPlanQuery {
    pub segment: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredCredentialPlanQuery {
    pub fn matches(&self, item: &TieredCredentialPlan) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredCredentialPlan]) -> Vec<TieredCredentialPlan> {
        let matched = collections::filter(items, |candidate: &TieredCredentialPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_credential_plans(
    store: Rc<dyn TieredCredentialPlanStore>,
    query: &TieredCredentialPlanQuery,
) -> Vec<TieredCredentialPlan> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredCredentialPlanStore> = new_memory_tiered_credential_plan_store();
        let item = new_tiered_credential_plan("code-400", "name-170");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
