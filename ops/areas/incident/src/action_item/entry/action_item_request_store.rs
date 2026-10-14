use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps action item requests by code.
pub trait ActionItemRequestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ActionItemRequest>;
    fn save(&self, item: &ActionItemRequest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ActionItemRequest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryActionItemRequestStore {
    pub items: RefCell<BTreeMap<String, ActionItemRequest>>,
    pub order: RefCell<Vec<String>>,
}

impl ActionItemRequestStore for MemoryActionItemRequestStore {
    fn find(&self, code: &str) -> Option<ActionItemRequest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ActionItemRequest) {
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

    fn list(&self) -> Vec<ActionItemRequest> {
        let mut out: Vec<ActionItemRequest> = Vec::new();
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

pub fn new_memory_action_item_request_store() -> Rc<dyn ActionItemRequestStore> {
    Rc::new(MemoryActionItemRequestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedActionItemRequestStore {
    pub inner: Rc<dyn ActionItemRequestStore>,
    pub cache: RefCell<cache::TtlCache<ActionItemRequest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ActionItemRequestStore for CachedActionItemRequestStore {
    fn find(&self, code: &str) -> Option<ActionItemRequest> {
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

    fn save(&self, item: &ActionItemRequest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ActionItemRequest> {
        self.inner.list()
    }
}

pub fn new_cached_action_item_request_store(
    inner: Rc<dyn ActionItemRequestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ActionItemRequestStore> {
    Rc::new(CachedActionItemRequestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ActionItemRequest>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects action item requests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRequestQuery {
    pub namespace: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ActionItemRequestQuery {
    pub fn matches(&self, item: &ActionItemRequest) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ActionItemRequest]) -> Vec<ActionItemRequest> {
        let matched = collections::filter(items, |candidate: &ActionItemRequest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_action_item_requests(
    store: Rc<dyn ActionItemRequestStore>,
    query: &ActionItemRequestQuery,
) -> Vec<ActionItemRequest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ActionItemRequestStore> = new_memory_action_item_request_store();
        let item = new_action_item_request("code-439", "label-222");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
