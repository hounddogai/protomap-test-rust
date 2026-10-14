use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped delegation drafts by code.
pub trait CappedDelegationDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedDelegationDraft>;
    fn save(&self, item: &CappedDelegationDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedDelegationDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedDelegationDraftStore {
    pub items: RefCell<BTreeMap<String, CappedDelegationDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedDelegationDraftStore for MemoryCappedDelegationDraftStore {
    fn find(&self, code: &str) -> Option<CappedDelegationDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedDelegationDraft) {
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

    fn list(&self) -> Vec<CappedDelegationDraft> {
        let mut out: Vec<CappedDelegationDraft> = Vec::new();
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

pub fn new_memory_capped_delegation_draft_store() -> Rc<dyn CappedDelegationDraftStore> {
    Rc::new(MemoryCappedDelegationDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedDelegationDraftStore {
    pub inner: Rc<dyn CappedDelegationDraftStore>,
    pub cache: RefCell<cache::TtlCache<CappedDelegationDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedDelegationDraftStore for CachedCappedDelegationDraftStore {
    fn find(&self, code: &str) -> Option<CappedDelegationDraft> {
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

    fn save(&self, item: &CappedDelegationDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedDelegationDraft> {
        self.inner.list()
    }
}

pub fn new_cached_capped_delegation_draft_store(
    inner: Rc<dyn CappedDelegationDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedDelegationDraftStore> {
    Rc::new(CachedCappedDelegationDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedDelegationDraft>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped delegation drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDelegationDraftQuery {
    pub category: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedDelegationDraftQuery {
    pub fn matches(&self, item: &CappedDelegationDraft) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedDelegationDraft]) -> Vec<CappedDelegationDraft> {
        let matched = collections::filter(items, |candidate: &CappedDelegationDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_delegation_drafts(
    store: Rc<dyn CappedDelegationDraftStore>,
    query: &CappedDelegationDraftQuery,
) -> Vec<CappedDelegationDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedDelegationDraftStore> = new_memory_capped_delegation_draft_store();
        let item = new_capped_delegation_draft("code-421", "label-620");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
