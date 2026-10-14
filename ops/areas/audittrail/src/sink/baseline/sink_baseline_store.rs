use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps sink baselines by code.
pub trait SinkBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SinkBaseline>;
    fn save(&self, item: &SinkBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SinkBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySinkBaselineStore {
    pub items: RefCell<BTreeMap<String, SinkBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl SinkBaselineStore for MemorySinkBaselineStore {
    fn find(&self, code: &str) -> Option<SinkBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SinkBaseline) {
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

    fn list(&self) -> Vec<SinkBaseline> {
        let mut out: Vec<SinkBaseline> = Vec::new();
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

pub fn new_memory_sink_baseline_store() -> Rc<dyn SinkBaselineStore> {
    Rc::new(MemorySinkBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSinkBaselineStore {
    pub inner: Rc<dyn SinkBaselineStore>,
    pub cache: RefCell<cache::TtlCache<SinkBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SinkBaselineStore for CachedSinkBaselineStore {
    fn find(&self, code: &str) -> Option<SinkBaseline> {
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

    fn save(&self, item: &SinkBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SinkBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_sink_baseline_store(
    inner: Rc<dyn SinkBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SinkBaselineStore> {
    Rc::new(CachedSinkBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SinkBaseline>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects sink baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkBaselineQuery {
    pub segment: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SinkBaselineQuery {
    pub fn matches(&self, item: &SinkBaseline) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
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

    pub fn apply(&self, items: &[SinkBaseline]) -> Vec<SinkBaseline> {
        let matched = collections::filter(items, |candidate: &SinkBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_sink_baselines(store: Rc<dyn SinkBaselineStore>, query: &SinkBaselineQuery) -> Vec<SinkBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SinkBaselineStore> = new_memory_sink_baseline_store();
        let item = new_sink_baseline("code-837", "name-843");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
