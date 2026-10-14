use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps sink records by code.
pub trait SinkRecordStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SinkRecord>;
    fn save(&self, item: &SinkRecord);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SinkRecord>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySinkRecordStore {
    pub items: RefCell<BTreeMap<String, SinkRecord>>,
    pub order: RefCell<Vec<String>>,
}

impl SinkRecordStore for MemorySinkRecordStore {
    fn find(&self, code: &str) -> Option<SinkRecord> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SinkRecord) {
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

    fn list(&self) -> Vec<SinkRecord> {
        let mut out: Vec<SinkRecord> = Vec::new();
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

pub fn new_memory_sink_record_store() -> Rc<dyn SinkRecordStore> {
    Rc::new(MemorySinkRecordStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSinkRecordStore {
    pub inner: Rc<dyn SinkRecordStore>,
    pub cache: RefCell<cache::TtlCache<SinkRecord>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SinkRecordStore for CachedSinkRecordStore {
    fn find(&self, code: &str) -> Option<SinkRecord> {
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

    fn save(&self, item: &SinkRecord) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SinkRecord> {
        self.inner.list()
    }
}

pub fn new_cached_sink_record_store(
    inner: Rc<dyn SinkRecordStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SinkRecordStore> {
    Rc::new(CachedSinkRecordStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SinkRecord>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects sink records. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkRecordQuery {
    pub title: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SinkRecordQuery {
    pub fn matches(&self, item: &SinkRecord) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SinkRecord]) -> Vec<SinkRecord> {
        let matched = collections::filter(items, |candidate: &SinkRecord| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_sink_records(store: Rc<dyn SinkRecordStore>, query: &SinkRecordQuery) -> Vec<SinkRecord> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SinkRecordStore> = new_memory_sink_record_store();
        let item = new_sink_record("code-687", "title-599");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
