use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling postmortem policies by code.
pub trait RollingPostmortemPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingPostmortemPolicy>;
    fn save(&self, item: &RollingPostmortemPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingPostmortemPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingPostmortemPolicyStore {
    pub items: RefCell<BTreeMap<String, RollingPostmortemPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingPostmortemPolicyStore for MemoryRollingPostmortemPolicyStore {
    fn find(&self, code: &str) -> Option<RollingPostmortemPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingPostmortemPolicy) {
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

    fn list(&self) -> Vec<RollingPostmortemPolicy> {
        let mut out: Vec<RollingPostmortemPolicy> = Vec::new();
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

pub fn new_memory_rolling_postmortem_policy_store() -> Rc<dyn RollingPostmortemPolicyStore> {
    Rc::new(MemoryRollingPostmortemPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingPostmortemPolicyStore {
    pub inner: Rc<dyn RollingPostmortemPolicyStore>,
    pub cache: RefCell<cache::TtlCache<RollingPostmortemPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingPostmortemPolicyStore for CachedRollingPostmortemPolicyStore {
    fn find(&self, code: &str) -> Option<RollingPostmortemPolicy> {
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

    fn save(&self, item: &RollingPostmortemPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingPostmortemPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_postmortem_policy_store(
    inner: Rc<dyn RollingPostmortemPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingPostmortemPolicyStore> {
    Rc::new(CachedRollingPostmortemPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingPostmortemPolicy>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling postmortem policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPostmortemPolicyQuery {
    pub label: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingPostmortemPolicyQuery {
    pub fn matches(&self, item: &RollingPostmortemPolicy) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingPostmortemPolicy]) -> Vec<RollingPostmortemPolicy> {
        let matched = collections::filter(items, |candidate: &RollingPostmortemPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_postmortem_policies(
    store: Rc<dyn RollingPostmortemPolicyStore>,
    query: &RollingPostmortemPolicyQuery,
) -> Vec<RollingPostmortemPolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingPostmortemPolicyStore> = new_memory_rolling_postmortem_policy_store();
        let item = new_rolling_postmortem_policy("code-389", "label-598");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
