use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps slo drafts by code.
pub trait SloDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SloDraft>;
    fn save(&self, item: &SloDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SloDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySloDraftStore {
    pub items: RefCell<BTreeMap<String, SloDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl SloDraftStore for MemorySloDraftStore {
    fn find(&self, code: &str) -> Option<SloDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SloDraft) {
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

    fn list(&self) -> Vec<SloDraft> {
        let mut out: Vec<SloDraft> = Vec::new();
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

pub fn new_memory_slo_draft_store() -> Rc<dyn SloDraftStore> {
    Rc::new(MemorySloDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSloDraftStore {
    pub inner: Rc<dyn SloDraftStore>,
    pub cache: RefCell<cache::TtlCache<SloDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SloDraftStore for CachedSloDraftStore {
    fn find(&self, code: &str) -> Option<SloDraft> {
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

    fn save(&self, item: &SloDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SloDraft> {
        self.inner.list()
    }
}

pub fn new_cached_slo_draft_store(
    inner: Rc<dyn SloDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SloDraftStore> {
    Rc::new(CachedSloDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SloDraft>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects slo drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SloDraftQuery {
    pub label: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SloDraftQuery {
    pub fn matches(&self, item: &SloDraft) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SloDraft]) -> Vec<SloDraft> {
        let matched = collections::filter(items, |candidate: &SloDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_slo_drafts(store: Rc<dyn SloDraftStore>, query: &SloDraftQuery) -> Vec<SloDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SloDraftStore> = new_memory_slo_draft_store();
        let item = new_slo_draft("code-849", "label-69");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
