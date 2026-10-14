use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict action item drafts by code.
pub trait StrictActionItemDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictActionItemDraft>;
    fn save(&self, item: &StrictActionItemDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictActionItemDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictActionItemDraftStore {
    pub items: RefCell<BTreeMap<String, StrictActionItemDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictActionItemDraftStore for MemoryStrictActionItemDraftStore {
    fn find(&self, code: &str) -> Option<StrictActionItemDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictActionItemDraft) {
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

    fn list(&self) -> Vec<StrictActionItemDraft> {
        let mut out: Vec<StrictActionItemDraft> = Vec::new();
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

pub fn new_memory_strict_action_item_draft_store() -> Rc<dyn StrictActionItemDraftStore> {
    Rc::new(MemoryStrictActionItemDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictActionItemDraftStore {
    pub inner: Rc<dyn StrictActionItemDraftStore>,
    pub cache: RefCell<cache::TtlCache<StrictActionItemDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictActionItemDraftStore for CachedStrictActionItemDraftStore {
    fn find(&self, code: &str) -> Option<StrictActionItemDraft> {
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

    fn save(&self, item: &StrictActionItemDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictActionItemDraft> {
        self.inner.list()
    }
}

pub fn new_cached_strict_action_item_draft_store(
    inner: Rc<dyn StrictActionItemDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictActionItemDraftStore> {
    Rc::new(CachedStrictActionItemDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictActionItemDraft>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict action item drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictActionItemDraftQuery {
    pub namespace: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictActionItemDraftQuery {
    pub fn matches(&self, item: &StrictActionItemDraft) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[StrictActionItemDraft]) -> Vec<StrictActionItemDraft> {
        let matched = collections::filter(items, |candidate: &StrictActionItemDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_action_item_drafts(
    store: Rc<dyn StrictActionItemDraftStore>,
    query: &StrictActionItemDraftQuery,
) -> Vec<StrictActionItemDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn StrictActionItemDraftStore> = new_memory_strict_action_item_draft_store();
        let item = new_strict_action_item_draft("code-424", "title-59");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
