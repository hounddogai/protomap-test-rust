use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional usage segments by code.
pub trait RegionalUsageSegmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalUsageSegment>;
    fn save(&self, item: &RegionalUsageSegment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalUsageSegment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalUsageSegmentStore {
    pub items: RefCell<BTreeMap<String, RegionalUsageSegment>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalUsageSegmentStore for MemoryRegionalUsageSegmentStore {
    fn find(&self, code: &str) -> Option<RegionalUsageSegment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalUsageSegment) {
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

    fn list(&self) -> Vec<RegionalUsageSegment> {
        let mut out: Vec<RegionalUsageSegment> = Vec::new();
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

pub fn new_memory_regional_usage_segment_store() -> Rc<dyn RegionalUsageSegmentStore> {
    Rc::new(MemoryRegionalUsageSegmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalUsageSegmentStore {
    pub inner: Rc<dyn RegionalUsageSegmentStore>,
    pub cache: RefCell<cache::TtlCache<RegionalUsageSegment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalUsageSegmentStore for CachedRegionalUsageSegmentStore {
    fn find(&self, code: &str) -> Option<RegionalUsageSegment> {
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

    fn save(&self, item: &RegionalUsageSegment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalUsageSegment> {
        self.inner.list()
    }
}

pub fn new_cached_regional_usage_segment_store(
    inner: Rc<dyn RegionalUsageSegmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalUsageSegmentStore> {
    Rc::new(CachedRegionalUsageSegmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalUsageSegment>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional usage segments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageSegmentQuery {
    pub name: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalUsageSegmentQuery {
    pub fn matches(&self, item: &RegionalUsageSegment) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalUsageSegment]) -> Vec<RegionalUsageSegment> {
        let matched = collections::filter(items, |candidate: &RegionalUsageSegment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_usage_segments(
    store: Rc<dyn RegionalUsageSegmentStore>,
    query: &RegionalUsageSegmentQuery,
) -> Vec<RegionalUsageSegment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalUsageSegmentStore> = new_memory_regional_usage_segment_store();
        let item = new_regional_usage_segment("code-100", "name-928");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
