use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global flow log checkpoints by code.
pub trait GlobalFlowLogCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalFlowLogCheckpoint>;
    fn save(&self, item: &GlobalFlowLogCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalFlowLogCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalFlowLogCheckpointStore {
    pub items: RefCell<BTreeMap<String, GlobalFlowLogCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalFlowLogCheckpointStore for MemoryGlobalFlowLogCheckpointStore {
    fn find(&self, code: &str) -> Option<GlobalFlowLogCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalFlowLogCheckpoint) {
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

    fn list(&self) -> Vec<GlobalFlowLogCheckpoint> {
        let mut out: Vec<GlobalFlowLogCheckpoint> = Vec::new();
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

pub fn new_memory_global_flow_log_checkpoint_store() -> Rc<dyn GlobalFlowLogCheckpointStore> {
    Rc::new(MemoryGlobalFlowLogCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalFlowLogCheckpointStore {
    pub inner: Rc<dyn GlobalFlowLogCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<GlobalFlowLogCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalFlowLogCheckpointStore for CachedGlobalFlowLogCheckpointStore {
    fn find(&self, code: &str) -> Option<GlobalFlowLogCheckpoint> {
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

    fn save(&self, item: &GlobalFlowLogCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalFlowLogCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_global_flow_log_checkpoint_store(
    inner: Rc<dyn GlobalFlowLogCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalFlowLogCheckpointStore> {
    Rc::new(CachedGlobalFlowLogCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalFlowLogCheckpoint>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global flow log checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFlowLogCheckpointQuery {
    pub label: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalFlowLogCheckpointQuery {
    pub fn matches(&self, item: &GlobalFlowLogCheckpoint) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalFlowLogCheckpoint]) -> Vec<GlobalFlowLogCheckpoint> {
        let matched = collections::filter(items, |candidate: &GlobalFlowLogCheckpoint| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_flow_log_checkpoints(
    store: Rc<dyn GlobalFlowLogCheckpointStore>,
    query: &GlobalFlowLogCheckpointQuery,
) -> Vec<GlobalFlowLogCheckpoint> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalFlowLogCheckpointStore> = new_memory_global_flow_log_checkpoint_store();
        let item = new_global_flow_log_checkpoint("code-795", "label-248");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
