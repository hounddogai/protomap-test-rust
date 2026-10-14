use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps sink targets by code.
pub trait SinkTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SinkTarget>;
    fn save(&self, item: &SinkTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SinkTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySinkTargetStore {
    pub items: RefCell<BTreeMap<String, SinkTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl SinkTargetStore for MemorySinkTargetStore {
    fn find(&self, code: &str) -> Option<SinkTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SinkTarget) {
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

    fn list(&self) -> Vec<SinkTarget> {
        let mut out: Vec<SinkTarget> = Vec::new();
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

pub fn new_memory_sink_target_store() -> Rc<dyn SinkTargetStore> {
    Rc::new(MemorySinkTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSinkTargetStore {
    pub inner: Rc<dyn SinkTargetStore>,
    pub cache: RefCell<cache::TtlCache<SinkTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SinkTargetStore for CachedSinkTargetStore {
    fn find(&self, code: &str) -> Option<SinkTarget> {
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

    fn save(&self, item: &SinkTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SinkTarget> {
        self.inner.list()
    }
}

pub fn new_cached_sink_target_store(
    inner: Rc<dyn SinkTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SinkTargetStore> {
    Rc::new(CachedSinkTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SinkTarget>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects sink targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkTargetQuery {
    pub country: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SinkTargetQuery {
    pub fn matches(&self, item: &SinkTarget) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SinkTarget]) -> Vec<SinkTarget> {
        let matched = collections::filter(items, |candidate: &SinkTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_sink_targets(store: Rc<dyn SinkTargetStore>, query: &SinkTargetQuery) -> Vec<SinkTarget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SinkTargetStore> = new_memory_sink_target_store();
        let item = new_sink_target("code-907", "title-82");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
