use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending runbook exceptions by code.
pub trait PendingRunbookExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingRunbookException>;
    fn save(&self, item: &PendingRunbookException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingRunbookException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingRunbookExceptionStore {
    pub items: RefCell<BTreeMap<String, PendingRunbookException>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingRunbookExceptionStore for MemoryPendingRunbookExceptionStore {
    fn find(&self, code: &str) -> Option<PendingRunbookException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingRunbookException) {
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

    fn list(&self) -> Vec<PendingRunbookException> {
        let mut out: Vec<PendingRunbookException> = Vec::new();
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

pub fn new_memory_pending_runbook_exception_store() -> Rc<dyn PendingRunbookExceptionStore> {
    Rc::new(MemoryPendingRunbookExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingRunbookExceptionStore {
    pub inner: Rc<dyn PendingRunbookExceptionStore>,
    pub cache: RefCell<cache::TtlCache<PendingRunbookException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingRunbookExceptionStore for CachedPendingRunbookExceptionStore {
    fn find(&self, code: &str) -> Option<PendingRunbookException> {
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

    fn save(&self, item: &PendingRunbookException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingRunbookException> {
        self.inner.list()
    }
}

pub fn new_cached_pending_runbook_exception_store(
    inner: Rc<dyn PendingRunbookExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingRunbookExceptionStore> {
    Rc::new(CachedPendingRunbookExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingRunbookException>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending runbook exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookExceptionQuery {
    pub name: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingRunbookExceptionQuery {
    pub fn matches(&self, item: &PendingRunbookException) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PendingRunbookException]) -> Vec<PendingRunbookException> {
        let matched = collections::filter(items, |candidate: &PendingRunbookException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_runbook_exceptions(
    store: Rc<dyn PendingRunbookExceptionStore>,
    query: &PendingRunbookExceptionQuery,
) -> Vec<PendingRunbookException> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PendingRunbookExceptionStore> = new_memory_pending_runbook_exception_store();
        let item = new_pending_runbook_exception("code-949", "name-961");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
