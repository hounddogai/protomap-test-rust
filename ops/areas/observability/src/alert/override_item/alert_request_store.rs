use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps alert requests by code.
pub trait AlertRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AlertRequest>;
    fn save(&self, item: &AlertRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AlertRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAlertRequestStore {
    pub items: RefCell<BTreeMap<String, AlertRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl AlertRequestStore for MemoryAlertRequestStore {
    fn find(&self, code: &str) -> Option<AlertRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AlertRequest) {
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

    fn list(&self) -> Vec<AlertRequest> {
        let mut out: Vec<AlertRequest> = Vec::new();
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

pub fn new_memory_alert_request_store() -> Rc<dyn AlertRequestStore> {
    Rc::new(MemoryAlertRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAlertRequestStore {
    pub inner: Rc<dyn AlertRequestStore>,
    pub cache: RefCell<cache::TtlCache<AlertRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AlertRequestStore for CachedAlertRequestStore {
    fn find(&self, code: &str) -> Option<AlertRequest> {
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

    fn save(&self, item: &AlertRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AlertRequest> {
        self.inner.list()
    }
}

pub fn new_cached_alert_request_store(
    inner: Rc<dyn AlertRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AlertRequestStore> {
    Rc::new(CachedAlertRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AlertRequest>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects alert requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertRequestQuery {
    pub segment: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AlertRequestQuery {
    pub fn matches(&self, item: &AlertRequest) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
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

    pub fn apply(&self, items: &[AlertRequest]) -> Vec<AlertRequest> {
        let matched = collections::filter(items, |candidate: &AlertRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_alert_requests(store: Rc<dyn AlertRequestStore>, query: &AlertRequestQuery) -> Vec<AlertRequest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AlertRequestStore> = new_memory_alert_request_store();
        let item = new_alert_request("code-131", "label-724");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
