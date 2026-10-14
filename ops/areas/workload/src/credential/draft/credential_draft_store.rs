use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps credential drafts by code.
pub trait CredentialDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CredentialDraft>;
    fn save(&self, item: &CredentialDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CredentialDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCredentialDraftStore {
    pub items: RefCell<BTreeMap<String, CredentialDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl CredentialDraftStore for MemoryCredentialDraftStore {
    fn find(&self, code: &str) -> Option<CredentialDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CredentialDraft) {
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

    fn list(&self) -> Vec<CredentialDraft> {
        let mut out: Vec<CredentialDraft> = Vec::new();
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

pub fn new_memory_credential_draft_store() -> Rc<dyn CredentialDraftStore> {
    Rc::new(MemoryCredentialDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCredentialDraftStore {
    pub inner: Rc<dyn CredentialDraftStore>,
    pub cache: RefCell<cache::TtlCache<CredentialDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CredentialDraftStore for CachedCredentialDraftStore {
    fn find(&self, code: &str) -> Option<CredentialDraft> {
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

    fn save(&self, item: &CredentialDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CredentialDraft> {
        self.inner.list()
    }
}

pub fn new_cached_credential_draft_store(
    inner: Rc<dyn CredentialDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CredentialDraftStore> {
    Rc::new(CachedCredentialDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CredentialDraft>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects credential drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialDraftQuery {
    pub category: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CredentialDraftQuery {
    pub fn matches(&self, item: &CredentialDraft) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CredentialDraft]) -> Vec<CredentialDraft> {
        let matched = collections::filter(items, |candidate: &CredentialDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_credential_drafts(
    store: Rc<dyn CredentialDraftStore>,
    query: &CredentialDraftQuery,
) -> Vec<CredentialDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CredentialDraftStore> = new_memory_credential_draft_store();
        let item = new_credential_draft("code-242", "name-451");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
