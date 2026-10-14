use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps sink drafts by code.
pub trait SinkDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SinkDraft>;
    fn save(&self, item: &SinkDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SinkDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySinkDraftStore {
    pub items: RefCell<BTreeMap<String, SinkDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl SinkDraftStore for MemorySinkDraftStore {
    fn find(&self, code: &str) -> Option<SinkDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SinkDraft) {
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

    fn list(&self) -> Vec<SinkDraft> {
        let mut out: Vec<SinkDraft> = Vec::new();
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

pub fn new_memory_sink_draft_store() -> Rc<dyn SinkDraftStore> {
    Rc::new(MemorySinkDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSinkDraftStore {
    pub inner: Rc<dyn SinkDraftStore>,
    pub cache: RefCell<cache::TtlCache<SinkDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SinkDraftStore for CachedSinkDraftStore {
    fn find(&self, code: &str) -> Option<SinkDraft> {
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

    fn save(&self, item: &SinkDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SinkDraft> {
        self.inner.list()
    }
}

pub fn new_cached_sink_draft_store(
    inner: Rc<dyn SinkDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SinkDraftStore> {
    Rc::new(CachedSinkDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SinkDraft>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects sink drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkDraftQuery {
    pub cluster: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SinkDraftQuery {
    pub fn matches(&self, item: &SinkDraft) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SinkDraft]) -> Vec<SinkDraft> {
        let matched = collections::filter(items, |candidate: &SinkDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_sink_drafts(store: Rc<dyn SinkDraftStore>, query: &SinkDraftQuery) -> Vec<SinkDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SinkDraftStore> = new_memory_sink_draft_store();
        let item = new_sink_draft("code-519", "name-879");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
