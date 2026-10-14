use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal egress exceptions by code.
pub trait SeasonalEgressExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalEgressException>;
    fn save(&self, item: &SeasonalEgressException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalEgressException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalEgressExceptionStore {
    pub items: RefCell<BTreeMap<String, SeasonalEgressException>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalEgressExceptionStore for MemorySeasonalEgressExceptionStore {
    fn find(&self, code: &str) -> Option<SeasonalEgressException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalEgressException) {
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

    fn list(&self) -> Vec<SeasonalEgressException> {
        let mut out: Vec<SeasonalEgressException> = Vec::new();
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

pub fn new_memory_seasonal_egress_exception_store() -> Rc<dyn SeasonalEgressExceptionStore> {
    Rc::new(MemorySeasonalEgressExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalEgressExceptionStore {
    pub inner: Rc<dyn SeasonalEgressExceptionStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalEgressException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalEgressExceptionStore for CachedSeasonalEgressExceptionStore {
    fn find(&self, code: &str) -> Option<SeasonalEgressException> {
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

    fn save(&self, item: &SeasonalEgressException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalEgressException> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_egress_exception_store(
    inner: Rc<dyn SeasonalEgressExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalEgressExceptionStore> {
    Rc::new(CachedSeasonalEgressExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalEgressException>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal egress exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressExceptionQuery {
    pub source: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalEgressExceptionQuery {
    pub fn matches(&self, item: &SeasonalEgressException) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalEgressException]) -> Vec<SeasonalEgressException> {
        let matched = collections::filter(items, |candidate: &SeasonalEgressException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_egress_exceptions(
    store: Rc<dyn SeasonalEgressExceptionStore>,
    query: &SeasonalEgressExceptionQuery,
) -> Vec<SeasonalEgressException> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalEgressExceptionStore> = new_memory_seasonal_egress_exception_store();
        let item = new_seasonal_egress_exception("code-71", "name-996");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
