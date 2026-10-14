use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict flow log checkpoints by code.
pub trait StrictFlowLogCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictFlowLogCheckpoint>;
    fn save(&self, item: &StrictFlowLogCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictFlowLogCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictFlowLogCheckpointStore {
    pub items: RefCell<BTreeMap<String, StrictFlowLogCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictFlowLogCheckpointStore for MemoryStrictFlowLogCheckpointStore {
    fn find(&self, code: &str) -> Option<StrictFlowLogCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictFlowLogCheckpoint) {
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

    fn list(&self) -> Vec<StrictFlowLogCheckpoint> {
        let mut out: Vec<StrictFlowLogCheckpoint> = Vec::new();
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

pub fn new_memory_strict_flow_log_checkpoint_store() -> Rc<dyn StrictFlowLogCheckpointStore> {
    Rc::new(MemoryStrictFlowLogCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictFlowLogCheckpointStore {
    pub inner: Rc<dyn StrictFlowLogCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<StrictFlowLogCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictFlowLogCheckpointStore for CachedStrictFlowLogCheckpointStore {
    fn find(&self, code: &str) -> Option<StrictFlowLogCheckpoint> {
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

    fn save(&self, item: &StrictFlowLogCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictFlowLogCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_strict_flow_log_checkpoint_store(
    inner: Rc<dyn StrictFlowLogCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictFlowLogCheckpointStore> {
    Rc::new(CachedStrictFlowLogCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictFlowLogCheckpoint>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict flow log checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFlowLogCheckpointQuery {
    pub category: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictFlowLogCheckpointQuery {
    pub fn matches(&self, item: &StrictFlowLogCheckpoint) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[StrictFlowLogCheckpoint]) -> Vec<StrictFlowLogCheckpoint> {
        let matched = collections::filter(items, |candidate: &StrictFlowLogCheckpoint| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_flow_log_checkpoints(
    store: Rc<dyn StrictFlowLogCheckpointStore>,
    query: &StrictFlowLogCheckpointQuery,
) -> Vec<StrictFlowLogCheckpoint> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn StrictFlowLogCheckpointStore> = new_memory_strict_flow_log_checkpoint_store();
        let item = new_strict_flow_log_checkpoint("code-270", "title-178");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
