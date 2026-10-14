use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps binding requests by code.
pub trait BindingRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BindingRequest>;
    fn save(&self, item: &BindingRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BindingRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBindingRequestStore {
    pub items: RefCell<BTreeMap<String, BindingRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl BindingRequestStore for MemoryBindingRequestStore {
    fn find(&self, code: &str) -> Option<BindingRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BindingRequest) {
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

    fn list(&self) -> Vec<BindingRequest> {
        let mut out: Vec<BindingRequest> = Vec::new();
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

pub fn new_memory_binding_request_store() -> Rc<dyn BindingRequestStore> {
    Rc::new(MemoryBindingRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBindingRequestStore {
    pub inner: Rc<dyn BindingRequestStore>,
    pub cache: RefCell<cache::TtlCache<BindingRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BindingRequestStore for CachedBindingRequestStore {
    fn find(&self, code: &str) -> Option<BindingRequest> {
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

    fn save(&self, item: &BindingRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BindingRequest> {
        self.inner.list()
    }
}

pub fn new_cached_binding_request_store(
    inner: Rc<dyn BindingRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BindingRequestStore> {
    Rc::new(CachedBindingRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BindingRequest>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects binding requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingRequestQuery {
    pub title: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BindingRequestQuery {
    pub fn matches(&self, item: &BindingRequest) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BindingRequest]) -> Vec<BindingRequest> {
        let matched = collections::filter(items, |candidate: &BindingRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_binding_requests(store: Rc<dyn BindingRequestStore>, query: &BindingRequestQuery) -> Vec<BindingRequest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BindingRequestStore> = new_memory_binding_request_store();
        let item = new_binding_request("code-452", "title-209");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
