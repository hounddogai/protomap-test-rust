use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient flow log entries by code.
pub trait LenientFlowLogEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientFlowLogEntry>;
    fn save(&self, item: &LenientFlowLogEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientFlowLogEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientFlowLogEntryStore {
    pub items: RefCell<BTreeMap<String, LenientFlowLogEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientFlowLogEntryStore for MemoryLenientFlowLogEntryStore {
    fn find(&self, code: &str) -> Option<LenientFlowLogEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientFlowLogEntry) {
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

    fn list(&self) -> Vec<LenientFlowLogEntry> {
        let mut out: Vec<LenientFlowLogEntry> = Vec::new();
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

pub fn new_memory_lenient_flow_log_entry_store() -> Rc<dyn LenientFlowLogEntryStore> {
    Rc::new(MemoryLenientFlowLogEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientFlowLogEntryStore {
    pub inner: Rc<dyn LenientFlowLogEntryStore>,
    pub cache: RefCell<cache::TtlCache<LenientFlowLogEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientFlowLogEntryStore for CachedLenientFlowLogEntryStore {
    fn find(&self, code: &str) -> Option<LenientFlowLogEntry> {
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

    fn save(&self, item: &LenientFlowLogEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientFlowLogEntry> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_flow_log_entry_store(
    inner: Rc<dyn LenientFlowLogEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientFlowLogEntryStore> {
    Rc::new(CachedLenientFlowLogEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientFlowLogEntry>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient flow log entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogEntryQuery {
    pub label: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientFlowLogEntryQuery {
    pub fn matches(&self, item: &LenientFlowLogEntry) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[LenientFlowLogEntry]) -> Vec<LenientFlowLogEntry> {
        let matched = collections::filter(items, |candidate: &LenientFlowLogEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_flow_log_entries(
    store: Rc<dyn LenientFlowLogEntryStore>,
    query: &LenientFlowLogEntryQuery,
) -> Vec<LenientFlowLogEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientFlowLogEntryStore> = new_memory_lenient_flow_log_entry_store();
        let item = new_lenient_flow_log_entry("code-380", "label-231");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
