use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling approval drafts by code.
pub trait RollingApprovalDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingApprovalDraft>;
    fn save(&self, item: &RollingApprovalDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingApprovalDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingApprovalDraftStore {
    pub items: RefCell<BTreeMap<String, RollingApprovalDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingApprovalDraftStore for MemoryRollingApprovalDraftStore {
    fn find(&self, code: &str) -> Option<RollingApprovalDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingApprovalDraft) {
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

    fn list(&self) -> Vec<RollingApprovalDraft> {
        let mut out: Vec<RollingApprovalDraft> = Vec::new();
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

pub fn new_memory_rolling_approval_draft_store() -> Rc<dyn RollingApprovalDraftStore> {
    Rc::new(MemoryRollingApprovalDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingApprovalDraftStore {
    pub inner: Rc<dyn RollingApprovalDraftStore>,
    pub cache: RefCell<cache::TtlCache<RollingApprovalDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingApprovalDraftStore for CachedRollingApprovalDraftStore {
    fn find(&self, code: &str) -> Option<RollingApprovalDraft> {
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

    fn save(&self, item: &RollingApprovalDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingApprovalDraft> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_approval_draft_store(
    inner: Rc<dyn RollingApprovalDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingApprovalDraftStore> {
    Rc::new(CachedRollingApprovalDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingApprovalDraft>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling approval drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalDraftQuery {
    pub title: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingApprovalDraftQuery {
    pub fn matches(&self, item: &RollingApprovalDraft) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingApprovalDraft]) -> Vec<RollingApprovalDraft> {
        let matched = collections::filter(items, |candidate: &RollingApprovalDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_approval_drafts(
    store: Rc<dyn RollingApprovalDraftStore>,
    query: &RollingApprovalDraftQuery,
) -> Vec<RollingApprovalDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingApprovalDraftStore> = new_memory_rolling_approval_draft_store();
        let item = new_rolling_approval_draft("code-145", "title-563");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
