use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped failover checkpoints by code.
pub trait CappedFailoverCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedFailoverCheckpoint>;
    fn save(&self, item: &CappedFailoverCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedFailoverCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedFailoverCheckpointStore {
    pub items: RefCell<BTreeMap<String, CappedFailoverCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedFailoverCheckpointStore for MemoryCappedFailoverCheckpointStore {
    fn find(&self, code: &str) -> Option<CappedFailoverCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedFailoverCheckpoint) {
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

    fn list(&self) -> Vec<CappedFailoverCheckpoint> {
        let mut out: Vec<CappedFailoverCheckpoint> = Vec::new();
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

pub fn new_memory_capped_failover_checkpoint_store() -> Rc<dyn CappedFailoverCheckpointStore> {
    Rc::new(MemoryCappedFailoverCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedFailoverCheckpointStore {
    pub inner: Rc<dyn CappedFailoverCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<CappedFailoverCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedFailoverCheckpointStore for CachedCappedFailoverCheckpointStore {
    fn find(&self, code: &str) -> Option<CappedFailoverCheckpoint> {
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

    fn save(&self, item: &CappedFailoverCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedFailoverCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_capped_failover_checkpoint_store(
    inner: Rc<dyn CappedFailoverCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedFailoverCheckpointStore> {
    Rc::new(CachedCappedFailoverCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedFailoverCheckpoint>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped failover checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedFailoverCheckpointQuery {
    pub region: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedFailoverCheckpointQuery {
    pub fn matches(&self, item: &CappedFailoverCheckpoint) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[CappedFailoverCheckpoint]) -> Vec<CappedFailoverCheckpoint> {
        let matched = collections::filter(items, |candidate: &CappedFailoverCheckpoint| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_failover_checkpoints(
    store: Rc<dyn CappedFailoverCheckpointStore>,
    query: &CappedFailoverCheckpointQuery,
) -> Vec<CappedFailoverCheckpoint> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedFailoverCheckpointStore> = new_memory_capped_failover_checkpoint_store();
        let item = new_capped_failover_checkpoint("code-748", "name-507");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
