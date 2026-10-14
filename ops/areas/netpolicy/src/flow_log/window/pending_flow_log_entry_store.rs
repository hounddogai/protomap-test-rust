use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending flow log entries by code.
pub trait PendingFlowLogEntryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingFlowLogEntry>;
    fn save(&self, item: &PendingFlowLogEntry);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingFlowLogEntry>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingFlowLogEntryStore {
    pub items: RefCell<BTreeMap<String, PendingFlowLogEntry>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingFlowLogEntryStore for MemoryPendingFlowLogEntryStore {
    fn find(&self, code: &str) -> Option<PendingFlowLogEntry> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingFlowLogEntry) {
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

    fn list(&self) -> Vec<PendingFlowLogEntry> {
        let mut out: Vec<PendingFlowLogEntry> = Vec::new();
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

pub fn new_memory_pending_flow_log_entry_store() -> Rc<dyn PendingFlowLogEntryStore> {
    Rc::new(MemoryPendingFlowLogEntryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingFlowLogEntryStore {
    pub inner: Rc<dyn PendingFlowLogEntryStore>,
    pub cache: RefCell<cache::TtlCache<PendingFlowLogEntry>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingFlowLogEntryStore for CachedPendingFlowLogEntryStore {
    fn find(&self, code: &str) -> Option<PendingFlowLogEntry> {
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

    fn save(&self, item: &PendingFlowLogEntry) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingFlowLogEntry> {
        self.inner.list()
    }
}

pub fn new_cached_pending_flow_log_entry_store(
    inner: Rc<dyn PendingFlowLogEntryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingFlowLogEntryStore> {
    Rc::new(CachedPendingFlowLogEntryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingFlowLogEntry>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending flow log entries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogEntryQuery {
    pub namespace: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingFlowLogEntryQuery {
    pub fn matches(&self, item: &PendingFlowLogEntry) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PendingFlowLogEntry]) -> Vec<PendingFlowLogEntry> {
        let matched = collections::filter(items, |candidate: &PendingFlowLogEntry| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_flow_log_entries(
    store: Rc<dyn PendingFlowLogEntryStore>,
    query: &PendingFlowLogEntryQuery,
) -> Vec<PendingFlowLogEntry> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PendingFlowLogEntryStore> = new_memory_pending_flow_log_entry_store();
        let item = new_pending_flow_log_entry("code-377", "label-518");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
