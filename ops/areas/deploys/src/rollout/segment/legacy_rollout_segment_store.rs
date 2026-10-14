use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy rollout segments by code.
pub trait LegacyRolloutSegmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyRolloutSegment>;
    fn save(&self, item: &LegacyRolloutSegment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyRolloutSegment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyRolloutSegmentStore {
    pub items: RefCell<BTreeMap<String, LegacyRolloutSegment>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyRolloutSegmentStore for MemoryLegacyRolloutSegmentStore {
    fn find(&self, code: &str) -> Option<LegacyRolloutSegment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyRolloutSegment) {
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

    fn list(&self) -> Vec<LegacyRolloutSegment> {
        let mut out: Vec<LegacyRolloutSegment> = Vec::new();
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

pub fn new_memory_legacy_rollout_segment_store() -> Rc<dyn LegacyRolloutSegmentStore> {
    Rc::new(MemoryLegacyRolloutSegmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyRolloutSegmentStore {
    pub inner: Rc<dyn LegacyRolloutSegmentStore>,
    pub cache: RefCell<cache::TtlCache<LegacyRolloutSegment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyRolloutSegmentStore for CachedLegacyRolloutSegmentStore {
    fn find(&self, code: &str) -> Option<LegacyRolloutSegment> {
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

    fn save(&self, item: &LegacyRolloutSegment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyRolloutSegment> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_rollout_segment_store(
    inner: Rc<dyn LegacyRolloutSegmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyRolloutSegmentStore> {
    Rc::new(CachedLegacyRolloutSegmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyRolloutSegment>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy rollout segments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutSegmentQuery {
    pub country: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyRolloutSegmentQuery {
    pub fn matches(&self, item: &LegacyRolloutSegment) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
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

    pub fn apply(&self, items: &[LegacyRolloutSegment]) -> Vec<LegacyRolloutSegment> {
        let matched = collections::filter(items, |candidate: &LegacyRolloutSegment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_rollout_segments(
    store: Rc<dyn LegacyRolloutSegmentStore>,
    query: &LegacyRolloutSegmentQuery,
) -> Vec<LegacyRolloutSegment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyRolloutSegmentStore> = new_memory_legacy_rollout_segment_store();
        let item = new_legacy_rollout_segment("code-84", "title-29");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
