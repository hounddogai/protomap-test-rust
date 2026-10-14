use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps suspension segments by code.
pub trait SuspensionSegmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SuspensionSegment>;
    fn save(&self, item: &SuspensionSegment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SuspensionSegment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySuspensionSegmentStore {
    pub items: RefCell<BTreeMap<String, SuspensionSegment>>,
    pub order: RefCell<Vec<String>>,
}

impl SuspensionSegmentStore for MemorySuspensionSegmentStore {
    fn find(&self, code: &str) -> Option<SuspensionSegment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SuspensionSegment) {
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

    fn list(&self) -> Vec<SuspensionSegment> {
        let mut out: Vec<SuspensionSegment> = Vec::new();
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

pub fn new_memory_suspension_segment_store() -> Rc<dyn SuspensionSegmentStore> {
    Rc::new(MemorySuspensionSegmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSuspensionSegmentStore {
    pub inner: Rc<dyn SuspensionSegmentStore>,
    pub cache: RefCell<cache::TtlCache<SuspensionSegment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SuspensionSegmentStore for CachedSuspensionSegmentStore {
    fn find(&self, code: &str) -> Option<SuspensionSegment> {
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

    fn save(&self, item: &SuspensionSegment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SuspensionSegment> {
        self.inner.list()
    }
}

pub fn new_cached_suspension_segment_store(
    inner: Rc<dyn SuspensionSegmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SuspensionSegmentStore> {
    Rc::new(CachedSuspensionSegmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SuspensionSegment>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects suspension segments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSegmentQuery {
    pub name: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SuspensionSegmentQuery {
    pub fn matches(&self, item: &SuspensionSegment) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SuspensionSegment]) -> Vec<SuspensionSegment> {
        let matched = collections::filter(items, |candidate: &SuspensionSegment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_suspension_segments(
    store: Rc<dyn SuspensionSegmentStore>,
    query: &SuspensionSegmentQuery,
) -> Vec<SuspensionSegment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SuspensionSegmentStore> = new_memory_suspension_segment_store();
        let item = new_suspension_segment("code-746", "name-993");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
