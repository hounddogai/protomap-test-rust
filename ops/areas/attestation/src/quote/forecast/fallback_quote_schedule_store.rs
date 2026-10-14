use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback quote schedules by code.
pub trait FallbackQuoteScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackQuoteSchedule>;
    fn save(&self, item: &FallbackQuoteSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackQuoteSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackQuoteScheduleStore {
    pub items: RefCell<BTreeMap<String, FallbackQuoteSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackQuoteScheduleStore for MemoryFallbackQuoteScheduleStore {
    fn find(&self, code: &str) -> Option<FallbackQuoteSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackQuoteSchedule) {
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

    fn list(&self) -> Vec<FallbackQuoteSchedule> {
        let mut out: Vec<FallbackQuoteSchedule> = Vec::new();
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

pub fn new_memory_fallback_quote_schedule_store() -> Rc<dyn FallbackQuoteScheduleStore> {
    Rc::new(MemoryFallbackQuoteScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackQuoteScheduleStore {
    pub inner: Rc<dyn FallbackQuoteScheduleStore>,
    pub cache: RefCell<cache::TtlCache<FallbackQuoteSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackQuoteScheduleStore for CachedFallbackQuoteScheduleStore {
    fn find(&self, code: &str) -> Option<FallbackQuoteSchedule> {
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

    fn save(&self, item: &FallbackQuoteSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackQuoteSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_quote_schedule_store(
    inner: Rc<dyn FallbackQuoteScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackQuoteScheduleStore> {
    Rc::new(CachedFallbackQuoteScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackQuoteSchedule>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback quote schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteScheduleQuery {
    pub namespace: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackQuoteScheduleQuery {
    pub fn matches(&self, item: &FallbackQuoteSchedule) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[FallbackQuoteSchedule]) -> Vec<FallbackQuoteSchedule> {
        let matched = collections::filter(items, |candidate: &FallbackQuoteSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_quote_schedules(
    store: Rc<dyn FallbackQuoteScheduleStore>,
    query: &FallbackQuoteScheduleQuery,
) -> Vec<FallbackQuoteSchedule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackQuoteScheduleStore> = new_memory_fallback_quote_schedule_store();
        let item = new_fallback_quote_schedule("code-833", "name-563");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
