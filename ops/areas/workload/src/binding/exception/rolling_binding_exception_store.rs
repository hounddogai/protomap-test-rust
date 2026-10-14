use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling binding exceptions by code.
pub trait RollingBindingExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingBindingException>;
    fn save(&self, item: &RollingBindingException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingBindingException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingBindingExceptionStore {
    pub items: RefCell<BTreeMap<String, RollingBindingException>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingBindingExceptionStore for MemoryRollingBindingExceptionStore {
    fn find(&self, code: &str) -> Option<RollingBindingException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingBindingException) {
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

    fn list(&self) -> Vec<RollingBindingException> {
        let mut out: Vec<RollingBindingException> = Vec::new();
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

pub fn new_memory_rolling_binding_exception_store() -> Rc<dyn RollingBindingExceptionStore> {
    Rc::new(MemoryRollingBindingExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingBindingExceptionStore {
    pub inner: Rc<dyn RollingBindingExceptionStore>,
    pub cache: RefCell<cache::TtlCache<RollingBindingException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingBindingExceptionStore for CachedRollingBindingExceptionStore {
    fn find(&self, code: &str) -> Option<RollingBindingException> {
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

    fn save(&self, item: &RollingBindingException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingBindingException> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_binding_exception_store(
    inner: Rc<dyn RollingBindingExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingBindingExceptionStore> {
    Rc::new(CachedRollingBindingExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingBindingException>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling binding exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingBindingExceptionQuery {
    pub locale: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingBindingExceptionQuery {
    pub fn matches(&self, item: &RollingBindingException) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingBindingException]) -> Vec<RollingBindingException> {
        let matched = collections::filter(items, |candidate: &RollingBindingException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_binding_exceptions(
    store: Rc<dyn RollingBindingExceptionStore>,
    query: &RollingBindingExceptionQuery,
) -> Vec<RollingBindingException> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingBindingExceptionStore> = new_memory_rolling_binding_exception_store();
        let item = new_rolling_binding_exception("code-866", "label-625");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
