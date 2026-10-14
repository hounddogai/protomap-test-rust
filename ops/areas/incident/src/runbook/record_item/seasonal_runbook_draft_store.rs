use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal runbook drafts by code.
pub trait SeasonalRunbookDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalRunbookDraft>;
    fn save(&self, item: &SeasonalRunbookDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalRunbookDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalRunbookDraftStore {
    pub items: RefCell<BTreeMap<String, SeasonalRunbookDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalRunbookDraftStore for MemorySeasonalRunbookDraftStore {
    fn find(&self, code: &str) -> Option<SeasonalRunbookDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalRunbookDraft) {
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

    fn list(&self) -> Vec<SeasonalRunbookDraft> {
        let mut out: Vec<SeasonalRunbookDraft> = Vec::new();
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

pub fn new_memory_seasonal_runbook_draft_store() -> Rc<dyn SeasonalRunbookDraftStore> {
    Rc::new(MemorySeasonalRunbookDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalRunbookDraftStore {
    pub inner: Rc<dyn SeasonalRunbookDraftStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalRunbookDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalRunbookDraftStore for CachedSeasonalRunbookDraftStore {
    fn find(&self, code: &str) -> Option<SeasonalRunbookDraft> {
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

    fn save(&self, item: &SeasonalRunbookDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalRunbookDraft> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_runbook_draft_store(
    inner: Rc<dyn SeasonalRunbookDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalRunbookDraftStore> {
    Rc::new(CachedSeasonalRunbookDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalRunbookDraft>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal runbook drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRunbookDraftQuery {
    pub category: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalRunbookDraftQuery {
    pub fn matches(&self, item: &SeasonalRunbookDraft) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalRunbookDraft]) -> Vec<SeasonalRunbookDraft> {
        let matched = collections::filter(items, |candidate: &SeasonalRunbookDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_runbook_drafts(
    store: Rc<dyn SeasonalRunbookDraftStore>,
    query: &SeasonalRunbookDraftQuery,
) -> Vec<SeasonalRunbookDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalRunbookDraftStore> = new_memory_seasonal_runbook_draft_store();
        let item = new_seasonal_runbook_draft("code-38", "name-251");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
