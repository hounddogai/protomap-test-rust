use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps silence rollups by code.
pub trait SilenceRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SilenceRollup>;
    fn save(&self, item: &SilenceRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SilenceRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySilenceRollupStore {
    pub items: RefCell<BTreeMap<String, SilenceRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl SilenceRollupStore for MemorySilenceRollupStore {
    fn find(&self, code: &str) -> Option<SilenceRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SilenceRollup) {
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

    fn list(&self) -> Vec<SilenceRollup> {
        let mut out: Vec<SilenceRollup> = Vec::new();
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

pub fn new_memory_silence_rollup_store() -> Rc<dyn SilenceRollupStore> {
    Rc::new(MemorySilenceRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSilenceRollupStore {
    pub inner: Rc<dyn SilenceRollupStore>,
    pub cache: RefCell<cache::TtlCache<SilenceRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SilenceRollupStore for CachedSilenceRollupStore {
    fn find(&self, code: &str) -> Option<SilenceRollup> {
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

    fn save(&self, item: &SilenceRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SilenceRollup> {
        self.inner.list()
    }
}

pub fn new_cached_silence_rollup_store(
    inner: Rc<dyn SilenceRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SilenceRollupStore> {
    Rc::new(CachedSilenceRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SilenceRollup>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects silence rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceRollupQuery {
    pub segment: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SilenceRollupQuery {
    pub fn matches(&self, item: &SilenceRollup) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SilenceRollup]) -> Vec<SilenceRollup> {
        let matched = collections::filter(items, |candidate: &SilenceRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_silence_rollups(store: Rc<dyn SilenceRollupStore>, query: &SilenceRollupQuery) -> Vec<SilenceRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SilenceRollupStore> = new_memory_silence_rollup_store();
        let item = new_silence_rollup("code-642", "name-711");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
