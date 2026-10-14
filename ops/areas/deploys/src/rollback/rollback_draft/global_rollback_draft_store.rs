use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global rollback drafts by code.
pub trait GlobalRollbackDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalRollbackDraft>;
    fn save(&self, item: &GlobalRollbackDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalRollbackDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalRollbackDraftStore {
    pub items: RefCell<BTreeMap<String, GlobalRollbackDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalRollbackDraftStore for MemoryGlobalRollbackDraftStore {
    fn find(&self, code: &str) -> Option<GlobalRollbackDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalRollbackDraft) {
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

    fn list(&self) -> Vec<GlobalRollbackDraft> {
        let mut out: Vec<GlobalRollbackDraft> = Vec::new();
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

pub fn new_memory_global_rollback_draft_store() -> Rc<dyn GlobalRollbackDraftStore> {
    Rc::new(MemoryGlobalRollbackDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalRollbackDraftStore {
    pub inner: Rc<dyn GlobalRollbackDraftStore>,
    pub cache: RefCell<cache::TtlCache<GlobalRollbackDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalRollbackDraftStore for CachedGlobalRollbackDraftStore {
    fn find(&self, code: &str) -> Option<GlobalRollbackDraft> {
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

    fn save(&self, item: &GlobalRollbackDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalRollbackDraft> {
        self.inner.list()
    }
}

pub fn new_cached_global_rollback_draft_store(
    inner: Rc<dyn GlobalRollbackDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalRollbackDraftStore> {
    Rc::new(CachedGlobalRollbackDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalRollbackDraft>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global rollback drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRollbackDraftQuery {
    pub title: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalRollbackDraftQuery {
    pub fn matches(&self, item: &GlobalRollbackDraft) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.ceiling < self.min_ceiling {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalRollbackDraft]) -> Vec<GlobalRollbackDraft> {
        let matched = collections::filter(items, |candidate: &GlobalRollbackDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_rollback_drafts(
    store: Rc<dyn GlobalRollbackDraftStore>,
    query: &GlobalRollbackDraftQuery,
) -> Vec<GlobalRollbackDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalRollbackDraftStore> = new_memory_global_rollback_draft_store();
        let item = new_global_rollback_draft("code-680", "title-857");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
