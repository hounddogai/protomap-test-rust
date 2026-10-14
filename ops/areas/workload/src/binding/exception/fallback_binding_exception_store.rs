use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback binding exceptions by code.
pub trait FallbackBindingExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackBindingException>;
    fn save(&self, item: &FallbackBindingException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackBindingException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackBindingExceptionStore {
    pub items: RefCell<BTreeMap<String, FallbackBindingException>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackBindingExceptionStore for MemoryFallbackBindingExceptionStore {
    fn find(&self, code: &str) -> Option<FallbackBindingException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackBindingException) {
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

    fn list(&self) -> Vec<FallbackBindingException> {
        let mut out: Vec<FallbackBindingException> = Vec::new();
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

pub fn new_memory_fallback_binding_exception_store() -> Rc<dyn FallbackBindingExceptionStore> {
    Rc::new(MemoryFallbackBindingExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackBindingExceptionStore {
    pub inner: Rc<dyn FallbackBindingExceptionStore>,
    pub cache: RefCell<cache::TtlCache<FallbackBindingException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackBindingExceptionStore for CachedFallbackBindingExceptionStore {
    fn find(&self, code: &str) -> Option<FallbackBindingException> {
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

    fn save(&self, item: &FallbackBindingException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackBindingException> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_binding_exception_store(
    inner: Rc<dyn FallbackBindingExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackBindingExceptionStore> {
    Rc::new(CachedFallbackBindingExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackBindingException>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback binding exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingExceptionQuery {
    pub title: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackBindingExceptionQuery {
    pub fn matches(&self, item: &FallbackBindingException) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackBindingException]) -> Vec<FallbackBindingException> {
        let matched = collections::filter(items, |candidate: &FallbackBindingException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_binding_exceptions(
    store: Rc<dyn FallbackBindingExceptionStore>,
    query: &FallbackBindingExceptionQuery,
) -> Vec<FallbackBindingException> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackBindingExceptionStore> = new_memory_fallback_binding_exception_store();
        let item = new_fallback_binding_exception("code-192", "title-357");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
