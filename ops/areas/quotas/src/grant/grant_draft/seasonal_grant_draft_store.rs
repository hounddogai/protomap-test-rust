use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal grant drafts by code.
pub trait SeasonalGrantDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalGrantDraft>;
    fn save(&self, item: &SeasonalGrantDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalGrantDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalGrantDraftStore {
    pub items: RefCell<BTreeMap<String, SeasonalGrantDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalGrantDraftStore for MemorySeasonalGrantDraftStore {
    fn find(&self, code: &str) -> Option<SeasonalGrantDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalGrantDraft) {
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

    fn list(&self) -> Vec<SeasonalGrantDraft> {
        let mut out: Vec<SeasonalGrantDraft> = Vec::new();
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

pub fn new_memory_seasonal_grant_draft_store() -> Rc<dyn SeasonalGrantDraftStore> {
    Rc::new(MemorySeasonalGrantDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalGrantDraftStore {
    pub inner: Rc<dyn SeasonalGrantDraftStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalGrantDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalGrantDraftStore for CachedSeasonalGrantDraftStore {
    fn find(&self, code: &str) -> Option<SeasonalGrantDraft> {
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

    fn save(&self, item: &SeasonalGrantDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalGrantDraft> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_grant_draft_store(
    inner: Rc<dyn SeasonalGrantDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalGrantDraftStore> {
    Rc::new(CachedSeasonalGrantDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalGrantDraft>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal grant drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantDraftQuery {
    pub title: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalGrantDraftQuery {
    pub fn matches(&self, item: &SeasonalGrantDraft) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalGrantDraft]) -> Vec<SeasonalGrantDraft> {
        let matched = collections::filter(items, |candidate: &SeasonalGrantDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_grant_drafts(
    store: Rc<dyn SeasonalGrantDraftStore>,
    query: &SeasonalGrantDraftQuery,
) -> Vec<SeasonalGrantDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalGrantDraftStore> = new_memory_seasonal_grant_draft_store();
        let item = new_seasonal_grant_draft("code-948", "title-839");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
