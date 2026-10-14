use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled slo records by code.
pub trait ScheduledSloRecordStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledSloRecord>;
    fn save(&self, item: &ScheduledSloRecord);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledSloRecord>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledSloRecordStore {
    pub items: RefCell<BTreeMap<String, ScheduledSloRecord>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledSloRecordStore for MemoryScheduledSloRecordStore {
    fn find(&self, code: &str) -> Option<ScheduledSloRecord> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledSloRecord) {
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

    fn list(&self) -> Vec<ScheduledSloRecord> {
        let mut out: Vec<ScheduledSloRecord> = Vec::new();
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

pub fn new_memory_scheduled_slo_record_store() -> Rc<dyn ScheduledSloRecordStore> {
    Rc::new(MemoryScheduledSloRecordStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledSloRecordStore {
    pub inner: Rc<dyn ScheduledSloRecordStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledSloRecord>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledSloRecordStore for CachedScheduledSloRecordStore {
    fn find(&self, code: &str) -> Option<ScheduledSloRecord> {
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

    fn save(&self, item: &ScheduledSloRecord) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledSloRecord> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_slo_record_store(
    inner: Rc<dyn ScheduledSloRecordStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledSloRecordStore> {
    Rc::new(CachedScheduledSloRecordStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledSloRecord>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled slo records. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSloRecordQuery {
    pub name: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledSloRecordQuery {
    pub fn matches(&self, item: &ScheduledSloRecord) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledSloRecord]) -> Vec<ScheduledSloRecord> {
        let matched = collections::filter(items, |candidate: &ScheduledSloRecord| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_slo_records(
    store: Rc<dyn ScheduledSloRecordStore>,
    query: &ScheduledSloRecordQuery,
) -> Vec<ScheduledSloRecord> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ScheduledSloRecordStore> = new_memory_scheduled_slo_record_store();
        let item = new_scheduled_slo_record("code-684", "name-490");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
