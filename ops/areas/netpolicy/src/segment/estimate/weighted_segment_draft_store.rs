use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps weighted segment drafts by code.
pub trait WeightedSegmentDraftStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<WeightedSegmentDraft>;
    fn save(&self, item: &WeightedSegmentDraft);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<WeightedSegmentDraft>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWeightedSegmentDraftStore {
    pub items: RefCell<BTreeMap<String, WeightedSegmentDraft>>,
    pub order: RefCell<Vec<String>>,
}

impl WeightedSegmentDraftStore for MemoryWeightedSegmentDraftStore {
    fn find(&self, code: &str) -> Option<WeightedSegmentDraft> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &WeightedSegmentDraft) {
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

    fn list(&self) -> Vec<WeightedSegmentDraft> {
        let mut out: Vec<WeightedSegmentDraft> = Vec::new();
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

pub fn new_memory_weighted_segment_draft_store() -> Rc<dyn WeightedSegmentDraftStore> {
    Rc::new(MemoryWeightedSegmentDraftStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedWeightedSegmentDraftStore {
    pub inner: Rc<dyn WeightedSegmentDraftStore>,
    pub cache: RefCell<cache::TtlCache<WeightedSegmentDraft>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl WeightedSegmentDraftStore for CachedWeightedSegmentDraftStore {
    fn find(&self, code: &str) -> Option<WeightedSegmentDraft> {
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

    fn save(&self, item: &WeightedSegmentDraft) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<WeightedSegmentDraft> {
        self.inner.list()
    }
}

pub fn new_cached_weighted_segment_draft_store(
    inner: Rc<dyn WeightedSegmentDraftStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn WeightedSegmentDraftStore> {
    Rc::new(CachedWeightedSegmentDraftStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<WeightedSegmentDraft>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects weighted segment drafts. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentDraftQuery {
    pub label: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl WeightedSegmentDraftQuery {
    pub fn matches(&self, item: &WeightedSegmentDraft) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[WeightedSegmentDraft]) -> Vec<WeightedSegmentDraft> {
        let matched = collections::filter(items, |candidate: &WeightedSegmentDraft| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_weighted_segment_drafts(
    store: Rc<dyn WeightedSegmentDraftStore>,
    query: &WeightedSegmentDraftQuery,
) -> Vec<WeightedSegmentDraft> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn WeightedSegmentDraftStore> = new_memory_weighted_segment_draft_store();
        let item = new_weighted_segment_draft("code-776", "label-816");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
