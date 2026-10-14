use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps segment notices by code.
pub trait SegmentNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SegmentNotice>;
    fn save(&self, item: &SegmentNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SegmentNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySegmentNoticeStore {
    pub items: RefCell<BTreeMap<String, SegmentNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl SegmentNoticeStore for MemorySegmentNoticeStore {
    fn find(&self, code: &str) -> Option<SegmentNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SegmentNotice) {
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

    fn list(&self) -> Vec<SegmentNotice> {
        let mut out: Vec<SegmentNotice> = Vec::new();
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

pub fn new_memory_segment_notice_store() -> Rc<dyn SegmentNoticeStore> {
    Rc::new(MemorySegmentNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSegmentNoticeStore {
    pub inner: Rc<dyn SegmentNoticeStore>,
    pub cache: RefCell<cache::TtlCache<SegmentNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SegmentNoticeStore for CachedSegmentNoticeStore {
    fn find(&self, code: &str) -> Option<SegmentNotice> {
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

    fn save(&self, item: &SegmentNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SegmentNotice> {
        self.inner.list()
    }
}

pub fn new_cached_segment_notice_store(
    inner: Rc<dyn SegmentNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SegmentNoticeStore> {
    Rc::new(CachedSegmentNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SegmentNotice>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects segment notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentNoticeQuery {
    pub name: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SegmentNoticeQuery {
    pub fn matches(&self, item: &SegmentNotice) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.attempts < self.min_attempts {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SegmentNotice]) -> Vec<SegmentNotice> {
        let matched = collections::filter(items, |candidate: &SegmentNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_segment_notices(store: Rc<dyn SegmentNoticeStore>, query: &SegmentNoticeQuery) -> Vec<SegmentNotice> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SegmentNoticeStore> = new_memory_segment_notice_store();
        let item = new_segment_notice("code-1", "name-960");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
