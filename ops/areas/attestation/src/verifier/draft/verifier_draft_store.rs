use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps verifier drafts by code.
pub trait VerifierDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<VerifierDraft>;
    fn save(&self, item: &VerifierDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<VerifierDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryVerifierDraftStore {
    pub items: RefCell<BTreeMap<String, VerifierDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl VerifierDraftStore for MemoryVerifierDraftStore {
    fn find(&self, code: &str) -> Option<VerifierDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &VerifierDraft) {
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

    fn list(&self) -> Vec<VerifierDraft> {
        let mut out: Vec<VerifierDraft> = Vec::new();
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

pub fn new_memory_verifier_draft_store() -> Rc<dyn VerifierDraftStore> {
    Rc::new(MemoryVerifierDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedVerifierDraftStore {
    pub inner: Rc<dyn VerifierDraftStore>,
    pub cache: RefCell<cache::TtlCache<VerifierDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl VerifierDraftStore for CachedVerifierDraftStore {
    fn find(&self, code: &str) -> Option<VerifierDraft> {
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

    fn save(&self, item: &VerifierDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<VerifierDraft> {
        self.inner.list()
    }
}

pub fn new_cached_verifier_draft_store(
    inner: Rc<dyn VerifierDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn VerifierDraftStore> {
    Rc::new(CachedVerifierDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<VerifierDraft>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects verifier drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierDraftQuery {
    pub country: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl VerifierDraftQuery {
    pub fn matches(&self, item: &VerifierDraft) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[VerifierDraft]) -> Vec<VerifierDraft> {
        let matched = collections::filter(items, |candidate: &VerifierDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_verifier_drafts(store: Rc<dyn VerifierDraftStore>, query: &VerifierDraftQuery) -> Vec<VerifierDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn VerifierDraftStore> = new_memory_verifier_draft_store();
        let item = new_verifier_draft("code-239", "title-481");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
