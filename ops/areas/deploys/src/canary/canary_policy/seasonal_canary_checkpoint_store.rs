use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal canary checkpoints by code.
pub trait SeasonalCanaryCheckpointStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalCanaryCheckpoint>;
    fn save(&self, item: &SeasonalCanaryCheckpoint);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalCanaryCheckpoint>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalCanaryCheckpointStore {
    pub items: RefCell<BTreeMap<String, SeasonalCanaryCheckpoint>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalCanaryCheckpointStore for MemorySeasonalCanaryCheckpointStore {
    fn find(&self, code: &str) -> Option<SeasonalCanaryCheckpoint> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalCanaryCheckpoint) {
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

    fn list(&self) -> Vec<SeasonalCanaryCheckpoint> {
        let mut out: Vec<SeasonalCanaryCheckpoint> = Vec::new();
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

pub fn new_memory_seasonal_canary_checkpoint_store() -> Rc<dyn SeasonalCanaryCheckpointStore> {
    Rc::new(MemorySeasonalCanaryCheckpointStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalCanaryCheckpointStore {
    pub inner: Rc<dyn SeasonalCanaryCheckpointStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalCanaryCheckpoint>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalCanaryCheckpointStore for CachedSeasonalCanaryCheckpointStore {
    fn find(&self, code: &str) -> Option<SeasonalCanaryCheckpoint> {
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

    fn save(&self, item: &SeasonalCanaryCheckpoint) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalCanaryCheckpoint> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_canary_checkpoint_store(
    inner: Rc<dyn SeasonalCanaryCheckpointStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalCanaryCheckpointStore> {
    Rc::new(CachedSeasonalCanaryCheckpointStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalCanaryCheckpoint>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal canary checkpoints. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryCheckpointQuery {
    pub name: String,
    pub min_window_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalCanaryCheckpointQuery {
    pub fn matches(&self, item: &SeasonalCanaryCheckpoint) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.window_days < self.min_window_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalCanaryCheckpoint]) -> Vec<SeasonalCanaryCheckpoint> {
        let matched = collections::filter(items, |candidate: &SeasonalCanaryCheckpoint| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_canary_checkpoints(
    store: Rc<dyn SeasonalCanaryCheckpointStore>,
    query: &SeasonalCanaryCheckpointQuery,
) -> Vec<SeasonalCanaryCheckpoint> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalCanaryCheckpointStore> = new_memory_seasonal_canary_checkpoint_store();
        let item = new_seasonal_canary_checkpoint("code-209", "name-489");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
