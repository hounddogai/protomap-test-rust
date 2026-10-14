use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps alert policies by code.
pub trait AlertPolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AlertPolicy>;
    fn save(&self, item: &AlertPolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AlertPolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAlertPolicyStore {
    pub items: RefCell<BTreeMap<String, AlertPolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl AlertPolicyStore for MemoryAlertPolicyStore {
    fn find(&self, code: &str) -> Option<AlertPolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AlertPolicy) {
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

    fn list(&self) -> Vec<AlertPolicy> {
        let mut out: Vec<AlertPolicy> = Vec::new();
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

pub fn new_memory_alert_policy_store() -> Rc<dyn AlertPolicyStore> {
    Rc::new(MemoryAlertPolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAlertPolicyStore {
    pub inner: Rc<dyn AlertPolicyStore>,
    pub cache: RefCell<cache::TtlCache<AlertPolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AlertPolicyStore for CachedAlertPolicyStore {
    fn find(&self, code: &str) -> Option<AlertPolicy> {
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

    fn save(&self, item: &AlertPolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AlertPolicy> {
        self.inner.list()
    }
}

pub fn new_cached_alert_policy_store(
    inner: Rc<dyn AlertPolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AlertPolicyStore> {
    Rc::new(CachedAlertPolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AlertPolicy>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects alert policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertPolicyQuery {
    pub name: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AlertPolicyQuery {
    pub fn matches(&self, item: &AlertPolicy) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AlertPolicy]) -> Vec<AlertPolicy> {
        let matched = collections::filter(items, |candidate: &AlertPolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_alert_policies(store: Rc<dyn AlertPolicyStore>, query: &AlertPolicyQuery) -> Vec<AlertPolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AlertPolicyStore> = new_memory_alert_policy_store();
        let item = new_alert_policy("code-981", "name-452");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
