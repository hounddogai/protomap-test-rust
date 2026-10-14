use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps alert targets by code.
pub trait AlertTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AlertTarget>;
    fn save(&self, item: &AlertTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AlertTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAlertTargetStore {
    pub items: RefCell<BTreeMap<String, AlertTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl AlertTargetStore for MemoryAlertTargetStore {
    fn find(&self, code: &str) -> Option<AlertTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AlertTarget) {
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

    fn list(&self) -> Vec<AlertTarget> {
        let mut out: Vec<AlertTarget> = Vec::new();
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

pub fn new_memory_alert_target_store() -> Rc<dyn AlertTargetStore> {
    Rc::new(MemoryAlertTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAlertTargetStore {
    pub inner: Rc<dyn AlertTargetStore>,
    pub cache: RefCell<cache::TtlCache<AlertTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AlertTargetStore for CachedAlertTargetStore {
    fn find(&self, code: &str) -> Option<AlertTarget> {
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

    fn save(&self, item: &AlertTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AlertTarget> {
        self.inner.list()
    }
}

pub fn new_cached_alert_target_store(
    inner: Rc<dyn AlertTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AlertTargetStore> {
    Rc::new(CachedAlertTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AlertTarget>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects alert targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertTargetQuery {
    pub namespace: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AlertTargetQuery {
    pub fn matches(&self, item: &AlertTarget) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[AlertTarget]) -> Vec<AlertTarget> {
        let matched = collections::filter(items, |candidate: &AlertTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_alert_targets(store: Rc<dyn AlertTargetStore>, query: &AlertTargetQuery) -> Vec<AlertTarget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AlertTargetStore> = new_memory_alert_target_store();
        let item = new_alert_target("code-167", "label-856");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
