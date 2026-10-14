use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered approval drafts by code.
pub trait TieredApprovalDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredApprovalDraft>;
    fn save(&self, item: &TieredApprovalDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredApprovalDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredApprovalDraftStore {
    pub items: RefCell<BTreeMap<String, TieredApprovalDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredApprovalDraftStore for MemoryTieredApprovalDraftStore {
    fn find(&self, code: &str) -> Option<TieredApprovalDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredApprovalDraft) {
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

    fn list(&self) -> Vec<TieredApprovalDraft> {
        let mut out: Vec<TieredApprovalDraft> = Vec::new();
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

pub fn new_memory_tiered_approval_draft_store() -> Rc<dyn TieredApprovalDraftStore> {
    Rc::new(MemoryTieredApprovalDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredApprovalDraftStore {
    pub inner: Rc<dyn TieredApprovalDraftStore>,
    pub cache: RefCell<cache::TtlCache<TieredApprovalDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredApprovalDraftStore for CachedTieredApprovalDraftStore {
    fn find(&self, code: &str) -> Option<TieredApprovalDraft> {
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

    fn save(&self, item: &TieredApprovalDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredApprovalDraft> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_approval_draft_store(
    inner: Rc<dyn TieredApprovalDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredApprovalDraftStore> {
    Rc::new(CachedTieredApprovalDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredApprovalDraft>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered approval drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalDraftQuery {
    pub country: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredApprovalDraftQuery {
    pub fn matches(&self, item: &TieredApprovalDraft) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredApprovalDraft]) -> Vec<TieredApprovalDraft> {
        let matched = collections::filter(items, |candidate: &TieredApprovalDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_approval_drafts(
    store: Rc<dyn TieredApprovalDraftStore>,
    query: &TieredApprovalDraftQuery,
) -> Vec<TieredApprovalDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredApprovalDraftStore> = new_memory_tiered_approval_draft_store();
        let item = new_tiered_approval_draft("code-987", "title-990");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
