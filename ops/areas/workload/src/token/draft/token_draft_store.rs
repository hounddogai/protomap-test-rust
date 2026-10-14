use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps token drafts by code.
pub trait TokenDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TokenDraft>;
    fn save(&self, item: &TokenDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TokenDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTokenDraftStore {
    pub items: RefCell<BTreeMap<String, TokenDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl TokenDraftStore for MemoryTokenDraftStore {
    fn find(&self, code: &str) -> Option<TokenDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TokenDraft) {
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

    fn list(&self) -> Vec<TokenDraft> {
        let mut out: Vec<TokenDraft> = Vec::new();
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

pub fn new_memory_token_draft_store() -> Rc<dyn TokenDraftStore> {
    Rc::new(MemoryTokenDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTokenDraftStore {
    pub inner: Rc<dyn TokenDraftStore>,
    pub cache: RefCell<cache::TtlCache<TokenDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TokenDraftStore for CachedTokenDraftStore {
    fn find(&self, code: &str) -> Option<TokenDraft> {
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

    fn save(&self, item: &TokenDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TokenDraft> {
        self.inner.list()
    }
}

pub fn new_cached_token_draft_store(
    inner: Rc<dyn TokenDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TokenDraftStore> {
    Rc::new(CachedTokenDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TokenDraft>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects token drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenDraftQuery {
    pub scope: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TokenDraftQuery {
    pub fn matches(&self, item: &TokenDraft) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TokenDraft]) -> Vec<TokenDraft> {
        let matched = collections::filter(items, |candidate: &TokenDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_token_drafts(store: Rc<dyn TokenDraftStore>, query: &TokenDraftQuery) -> Vec<TokenDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TokenDraftStore> = new_memory_token_draft_store();
        let item = new_token_draft("code-123", "label-3");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
