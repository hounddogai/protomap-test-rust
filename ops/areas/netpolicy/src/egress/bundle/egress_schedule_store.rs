use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps egress schedules by code.
pub trait EgressScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EgressSchedule>;
    fn save(&self, item: &EgressSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EgressSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEgressScheduleStore {
    pub items: RefCell<BTreeMap<String, EgressSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl EgressScheduleStore for MemoryEgressScheduleStore {
    fn find(&self, code: &str) -> Option<EgressSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EgressSchedule) {
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

    fn list(&self) -> Vec<EgressSchedule> {
        let mut out: Vec<EgressSchedule> = Vec::new();
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

pub fn new_memory_egress_schedule_store() -> Rc<dyn EgressScheduleStore> {
    Rc::new(MemoryEgressScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEgressScheduleStore {
    pub inner: Rc<dyn EgressScheduleStore>,
    pub cache: RefCell<cache::TtlCache<EgressSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EgressScheduleStore for CachedEgressScheduleStore {
    fn find(&self, code: &str) -> Option<EgressSchedule> {
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

    fn save(&self, item: &EgressSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EgressSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_egress_schedule_store(
    inner: Rc<dyn EgressScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EgressScheduleStore> {
    Rc::new(CachedEgressScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EgressSchedule>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects egress schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressScheduleQuery {
    pub name: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EgressScheduleQuery {
    pub fn matches(&self, item: &EgressSchedule) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[EgressSchedule]) -> Vec<EgressSchedule> {
        let matched = collections::filter(items, |candidate: &EgressSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_egress_schedules(store: Rc<dyn EgressScheduleStore>, query: &EgressScheduleQuery) -> Vec<EgressSchedule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn EgressScheduleStore> = new_memory_egress_schedule_store();
        let item = new_egress_schedule("code-741", "name-542");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
