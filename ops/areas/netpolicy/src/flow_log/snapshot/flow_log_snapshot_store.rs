use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps flow log snapshots by code.
pub trait FlowLogSnapshotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FlowLogSnapshot>;
    fn save(&self, item: &FlowLogSnapshot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FlowLogSnapshot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFlowLogSnapshotStore {
    pub items: RefCell<BTreeMap<String, FlowLogSnapshot>>,
    pub order: RefCell<Vec<String>>,
}

impl FlowLogSnapshotStore for MemoryFlowLogSnapshotStore {
    fn find(&self, code: &str) -> Option<FlowLogSnapshot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FlowLogSnapshot) {
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

    fn list(&self) -> Vec<FlowLogSnapshot> {
        let mut out: Vec<FlowLogSnapshot> = Vec::new();
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

pub fn new_memory_flow_log_snapshot_store() -> Rc<dyn FlowLogSnapshotStore> {
    Rc::new(MemoryFlowLogSnapshotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFlowLogSnapshotStore {
    pub inner: Rc<dyn FlowLogSnapshotStore>,
    pub cache: RefCell<cache::TtlCache<FlowLogSnapshot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FlowLogSnapshotStore for CachedFlowLogSnapshotStore {
    fn find(&self, code: &str) -> Option<FlowLogSnapshot> {
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

    fn save(&self, item: &FlowLogSnapshot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FlowLogSnapshot> {
        self.inner.list()
    }
}

pub fn new_cached_flow_log_snapshot_store(
    inner: Rc<dyn FlowLogSnapshotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FlowLogSnapshotStore> {
    Rc::new(CachedFlowLogSnapshotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FlowLogSnapshot>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects flow log snapshots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogSnapshotQuery {
    pub segment: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FlowLogSnapshotQuery {
    pub fn matches(&self, item: &FlowLogSnapshot) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FlowLogSnapshot]) -> Vec<FlowLogSnapshot> {
        let matched = collections::filter(items, |candidate: &FlowLogSnapshot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_flow_log_snapshots(
    store: Rc<dyn FlowLogSnapshotStore>,
    query: &FlowLogSnapshotQuery,
) -> Vec<FlowLogSnapshot> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FlowLogSnapshotStore> = new_memory_flow_log_snapshot_store();
        let item = new_flow_log_snapshot("code-687", "title-309");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
