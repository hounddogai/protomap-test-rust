use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps silence exceptions by code.
pub trait SilenceExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SilenceException>;
    fn save(&self, item: &SilenceException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SilenceException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySilenceExceptionStore {
    pub items: RefCell<BTreeMap<String, SilenceException>>,
    pub order: RefCell<Vec<String>>,
}

impl SilenceExceptionStore for MemorySilenceExceptionStore {
    fn find(&self, code: &str) -> Option<SilenceException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SilenceException) {
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

    fn list(&self) -> Vec<SilenceException> {
        let mut out: Vec<SilenceException> = Vec::new();
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

pub fn new_memory_silence_exception_store() -> Rc<dyn SilenceExceptionStore> {
    Rc::new(MemorySilenceExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSilenceExceptionStore {
    pub inner: Rc<dyn SilenceExceptionStore>,
    pub cache: RefCell<cache::TtlCache<SilenceException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SilenceExceptionStore for CachedSilenceExceptionStore {
    fn find(&self, code: &str) -> Option<SilenceException> {
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

    fn save(&self, item: &SilenceException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SilenceException> {
        self.inner.list()
    }
}

pub fn new_cached_silence_exception_store(
    inner: Rc<dyn SilenceExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SilenceExceptionStore> {
    Rc::new(CachedSilenceExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SilenceException>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects silence exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceExceptionQuery {
    pub label: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SilenceExceptionQuery {
    pub fn matches(&self, item: &SilenceException) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SilenceException]) -> Vec<SilenceException> {
        let matched = collections::filter(items, |candidate: &SilenceException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_silence_exceptions(
    store: Rc<dyn SilenceExceptionStore>,
    query: &SilenceExceptionQuery,
) -> Vec<SilenceException> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SilenceExceptionStore> = new_memory_silence_exception_store();
        let item = new_silence_exception("code-328", "label-422");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
