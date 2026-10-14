use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps isolation drafts by code.
pub trait IsolationDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<IsolationDraft>;
    fn save(&self, item: &IsolationDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<IsolationDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryIsolationDraftStore {
    pub items: RefCell<BTreeMap<String, IsolationDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl IsolationDraftStore for MemoryIsolationDraftStore {
    fn find(&self, code: &str) -> Option<IsolationDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &IsolationDraft) {
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

    fn list(&self) -> Vec<IsolationDraft> {
        let mut out: Vec<IsolationDraft> = Vec::new();
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

pub fn new_memory_isolation_draft_store() -> Rc<dyn IsolationDraftStore> {
    Rc::new(MemoryIsolationDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedIsolationDraftStore {
    pub inner: Rc<dyn IsolationDraftStore>,
    pub cache: RefCell<cache::TtlCache<IsolationDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl IsolationDraftStore for CachedIsolationDraftStore {
    fn find(&self, code: &str) -> Option<IsolationDraft> {
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

    fn save(&self, item: &IsolationDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<IsolationDraft> {
        self.inner.list()
    }
}

pub fn new_cached_isolation_draft_store(
    inner: Rc<dyn IsolationDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn IsolationDraftStore> {
    Rc::new(CachedIsolationDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<IsolationDraft>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects isolation drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationDraftQuery {
    pub scope: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl IsolationDraftQuery {
    pub fn matches(&self, item: &IsolationDraft) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[IsolationDraft]) -> Vec<IsolationDraft> {
        let matched = collections::filter(items, |candidate: &IsolationDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_isolation_drafts(store: Rc<dyn IsolationDraftStore>, query: &IsolationDraftQuery) -> Vec<IsolationDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn IsolationDraftStore> = new_memory_isolation_draft_store();
        let item = new_isolation_draft("code-655", "label-972");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
