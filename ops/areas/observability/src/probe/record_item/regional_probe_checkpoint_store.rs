use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional probe checkpoints by code.
pub trait RegionalProbeCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalProbeCheckpoint>;
    fn save(&self, item: &RegionalProbeCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalProbeCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalProbeCheckpointStore {
    pub items: RefCell<BTreeMap<String, RegionalProbeCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalProbeCheckpointStore for MemoryRegionalProbeCheckpointStore {
    fn find(&self, code: &str) -> Option<RegionalProbeCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalProbeCheckpoint) {
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

    fn list(&self) -> Vec<RegionalProbeCheckpoint> {
        let mut out: Vec<RegionalProbeCheckpoint> = Vec::new();
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

pub fn new_memory_regional_probe_checkpoint_store() -> Rc<dyn RegionalProbeCheckpointStore> {
    Rc::new(MemoryRegionalProbeCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalProbeCheckpointStore {
    pub inner: Rc<dyn RegionalProbeCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<RegionalProbeCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalProbeCheckpointStore for CachedRegionalProbeCheckpointStore {
    fn find(&self, code: &str) -> Option<RegionalProbeCheckpoint> {
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

    fn save(&self, item: &RegionalProbeCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalProbeCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_regional_probe_checkpoint_store(
    inner: Rc<dyn RegionalProbeCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalProbeCheckpointStore> {
    Rc::new(CachedRegionalProbeCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalProbeCheckpoint>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional probe checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeCheckpointQuery {
    pub scope: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalProbeCheckpointQuery {
    pub fn matches(&self, item: &RegionalProbeCheckpoint) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalProbeCheckpoint]) -> Vec<RegionalProbeCheckpoint> {
        let matched = collections::filter(items, |candidate: &RegionalProbeCheckpoint| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_probe_checkpoints(
    store: Rc<dyn RegionalProbeCheckpointStore>,
    query: &RegionalProbeCheckpointQuery,
) -> Vec<RegionalProbeCheckpoint> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalProbeCheckpointStore> = new_memory_regional_probe_checkpoint_store();
        let item = new_regional_probe_checkpoint("code-268", "label-43");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
