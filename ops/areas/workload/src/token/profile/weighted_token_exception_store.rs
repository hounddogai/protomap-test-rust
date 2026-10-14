use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted token exceptions by code.
pub trait WeightedTokenExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedTokenException>;
    fn save(&self, item: &WeightedTokenException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedTokenException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedTokenExceptionStore {
    pub items: RefCell<BTreeMap<String, WeightedTokenException>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedTokenExceptionStore for MemoryWeightedTokenExceptionStore {
    fn find(&self, code: &str) -> Option<WeightedTokenException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedTokenException) {
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

    fn list(&self) -> Vec<WeightedTokenException> {
        let mut out: Vec<WeightedTokenException> = Vec::new();
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

pub fn new_memory_weighted_token_exception_store() -> Rc<dyn WeightedTokenExceptionStore> {
    Rc::new(MemoryWeightedTokenExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedTokenExceptionStore {
    pub inner: Rc<dyn WeightedTokenExceptionStore>,
    pub cache: RefCell<cache::TtlCache<WeightedTokenException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedTokenExceptionStore for CachedWeightedTokenExceptionStore {
    fn find(&self, code: &str) -> Option<WeightedTokenException> {
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

    fn save(&self, item: &WeightedTokenException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedTokenException> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_token_exception_store(
    inner: Rc<dyn WeightedTokenExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedTokenExceptionStore> {
    Rc::new(CachedWeightedTokenExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedTokenException>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted token exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenExceptionQuery {
    pub scope: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedTokenExceptionQuery {
    pub fn matches(&self, item: &WeightedTokenException) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.ends_at < self.min_ends_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedTokenException]) -> Vec<WeightedTokenException> {
        let matched = collections::filter(items, |candidate: &WeightedTokenException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_token_exceptions(
    store: Rc<dyn WeightedTokenExceptionStore>,
    query: &WeightedTokenExceptionQuery,
) -> Vec<WeightedTokenException> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedTokenExceptionStore> = new_memory_weighted_token_exception_store();
        let item = new_weighted_token_exception("code-280", "label-542");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
