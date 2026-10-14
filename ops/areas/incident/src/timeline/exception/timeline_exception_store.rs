use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps timeline exceptions by code.
pub trait TimelineExceptionStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TimelineException>;
    fn save(&self, item: &TimelineException);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TimelineException>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTimelineExceptionStore {
    pub items: RefCell<BTreeMap<String, TimelineException>>,
    pub order: RefCell<Vec<String>>,
}

impl TimelineExceptionStore for MemoryTimelineExceptionStore {
    fn find(&self, code: &str) -> Option<TimelineException> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TimelineException) {
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

    fn list(&self) -> Vec<TimelineException> {
        let mut out: Vec<TimelineException> = Vec::new();
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

pub fn new_memory_timeline_exception_store() -> Rc<dyn TimelineExceptionStore> {
    Rc::new(MemoryTimelineExceptionStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTimelineExceptionStore {
    pub inner: Rc<dyn TimelineExceptionStore>,
    pub cache: RefCell<cache::TtlCache<TimelineException>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TimelineExceptionStore for CachedTimelineExceptionStore {
    fn find(&self, code: &str) -> Option<TimelineException> {
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

    fn save(&self, item: &TimelineException) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TimelineException> {
        self.inner.list()
    }
}

pub fn new_cached_timeline_exception_store(
    inner: Rc<dyn TimelineExceptionStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TimelineExceptionStore> {
    Rc::new(CachedTimelineExceptionStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TimelineException>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects timeline exceptions. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineExceptionQuery {
    pub scope: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TimelineExceptionQuery {
    pub fn matches(&self, item: &TimelineException) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TimelineException]) -> Vec<TimelineException> {
        let matched = collections::filter(items, |candidate: &TimelineException| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_timeline_exceptions(
    store: Rc<dyn TimelineExceptionStore>,
    query: &TimelineExceptionQuery,
) -> Vec<TimelineException> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TimelineExceptionStore> = new_memory_timeline_exception_store();
        let item = new_timeline_exception("code-485", "title-44");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
