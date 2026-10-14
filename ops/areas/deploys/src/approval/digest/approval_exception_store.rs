use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps approval exceptions by code.
pub trait ApprovalExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ApprovalException>;
    fn save(&self, item: &ApprovalException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ApprovalException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryApprovalExceptionStore {
    pub items: RefCell<BTreeMap<String, ApprovalException>>,
    pub order: RefCell<Vec<String>>,
}

impl ApprovalExceptionStore for MemoryApprovalExceptionStore {
    fn find(&self, code: &str) -> Option<ApprovalException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ApprovalException) {
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

    fn list(&self) -> Vec<ApprovalException> {
        let mut out: Vec<ApprovalException> = Vec::new();
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

pub fn new_memory_approval_exception_store() -> Rc<dyn ApprovalExceptionStore> {
    Rc::new(MemoryApprovalExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedApprovalExceptionStore {
    pub inner: Rc<dyn ApprovalExceptionStore>,
    pub cache: RefCell<cache::TtlCache<ApprovalException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ApprovalExceptionStore for CachedApprovalExceptionStore {
    fn find(&self, code: &str) -> Option<ApprovalException> {
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

    fn save(&self, item: &ApprovalException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ApprovalException> {
        self.inner.list()
    }
}

pub fn new_cached_approval_exception_store(
    inner: Rc<dyn ApprovalExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ApprovalExceptionStore> {
    Rc::new(CachedApprovalExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ApprovalException>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects approval exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalExceptionQuery {
    pub country: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ApprovalExceptionQuery {
    pub fn matches(&self, item: &ApprovalException) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
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

    pub fn apply(&self, items: &[ApprovalException]) -> Vec<ApprovalException> {
        let matched = collections::filter(items, |candidate: &ApprovalException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_approval_exceptions(
    store: Rc<dyn ApprovalExceptionStore>,
    query: &ApprovalExceptionQuery,
) -> Vec<ApprovalException> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ApprovalExceptionStore> = new_memory_approval_exception_store();
        let item = new_approval_exception("code-987", "title-253");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
