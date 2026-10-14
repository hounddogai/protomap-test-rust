use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped audit event requests by code.
pub trait CappedAuditEventRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedAuditEventRequest>;
    fn save(&self, item: &CappedAuditEventRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedAuditEventRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedAuditEventRequestStore {
    pub items: RefCell<BTreeMap<String, CappedAuditEventRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedAuditEventRequestStore for MemoryCappedAuditEventRequestStore {
    fn find(&self, code: &str) -> Option<CappedAuditEventRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedAuditEventRequest) {
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

    fn list(&self) -> Vec<CappedAuditEventRequest> {
        let mut out: Vec<CappedAuditEventRequest> = Vec::new();
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

pub fn new_memory_capped_audit_event_request_store() -> Rc<dyn CappedAuditEventRequestStore> {
    Rc::new(MemoryCappedAuditEventRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedAuditEventRequestStore {
    pub inner: Rc<dyn CappedAuditEventRequestStore>,
    pub cache: RefCell<cache::TtlCache<CappedAuditEventRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedAuditEventRequestStore for CachedCappedAuditEventRequestStore {
    fn find(&self, code: &str) -> Option<CappedAuditEventRequest> {
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

    fn save(&self, item: &CappedAuditEventRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedAuditEventRequest> {
        self.inner.list()
    }
}

pub fn new_cached_capped_audit_event_request_store(
    inner: Rc<dyn CappedAuditEventRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedAuditEventRequestStore> {
    Rc::new(CachedCappedAuditEventRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedAuditEventRequest>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped audit event requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedAuditEventRequestQuery {
    pub name: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedAuditEventRequestQuery {
    pub fn matches(&self, item: &CappedAuditEventRequest) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedAuditEventRequest]) -> Vec<CappedAuditEventRequest> {
        let matched = collections::filter(items, |candidate: &CappedAuditEventRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_audit_event_requests(
    store: Rc<dyn CappedAuditEventRequestStore>,
    query: &CappedAuditEventRequestQuery,
) -> Vec<CappedAuditEventRequest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedAuditEventRequestStore> = new_memory_capped_audit_event_request_store();
        let item = new_capped_audit_event_request("code-693", "name-576");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
