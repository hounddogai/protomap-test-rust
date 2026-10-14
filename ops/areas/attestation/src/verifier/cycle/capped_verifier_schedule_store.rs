use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped verifier schedules by code.
pub trait CappedVerifierScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedVerifierSchedule>;
    fn save(&self, item: &CappedVerifierSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedVerifierSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedVerifierScheduleStore {
    pub items: RefCell<BTreeMap<String, CappedVerifierSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedVerifierScheduleStore for MemoryCappedVerifierScheduleStore {
    fn find(&self, code: &str) -> Option<CappedVerifierSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedVerifierSchedule) {
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

    fn list(&self) -> Vec<CappedVerifierSchedule> {
        let mut out: Vec<CappedVerifierSchedule> = Vec::new();
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

pub fn new_memory_capped_verifier_schedule_store() -> Rc<dyn CappedVerifierScheduleStore> {
    Rc::new(MemoryCappedVerifierScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedVerifierScheduleStore {
    pub inner: Rc<dyn CappedVerifierScheduleStore>,
    pub cache: RefCell<cache::TtlCache<CappedVerifierSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedVerifierScheduleStore for CachedCappedVerifierScheduleStore {
    fn find(&self, code: &str) -> Option<CappedVerifierSchedule> {
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

    fn save(&self, item: &CappedVerifierSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedVerifierSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_capped_verifier_schedule_store(
    inner: Rc<dyn CappedVerifierScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedVerifierScheduleStore> {
    Rc::new(CachedCappedVerifierScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedVerifierSchedule>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped verifier schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierScheduleQuery {
    pub label: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedVerifierScheduleQuery {
    pub fn matches(&self, item: &CappedVerifierSchedule) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedVerifierSchedule]) -> Vec<CappedVerifierSchedule> {
        let matched = collections::filter(items, |candidate: &CappedVerifierSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_verifier_schedules(
    store: Rc<dyn CappedVerifierScheduleStore>,
    query: &CappedVerifierScheduleQuery,
) -> Vec<CappedVerifierSchedule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedVerifierScheduleStore> = new_memory_capped_verifier_schedule_store();
        let item = new_capped_verifier_schedule("code-686", "label-229");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
