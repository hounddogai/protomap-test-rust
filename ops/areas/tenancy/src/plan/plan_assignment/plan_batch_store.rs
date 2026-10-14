use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps plan batches by code.
pub trait PlanBatchStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PlanBatch>;
    fn save(&self, item: &PlanBatch);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PlanBatch>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPlanBatchStore {
    pub items: RefCell<BTreeMap<String, PlanBatch>>,
    pub order: RefCell<Vec<String>>,
}

impl PlanBatchStore for MemoryPlanBatchStore {
    fn find(&self, code: &str) -> Option<PlanBatch> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PlanBatch) {
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

    fn list(&self) -> Vec<PlanBatch> {
        let mut out: Vec<PlanBatch> = Vec::new();
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

pub fn new_memory_plan_batch_store() -> Rc<dyn PlanBatchStore> {
    Rc::new(MemoryPlanBatchStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPlanBatchStore {
    pub inner: Rc<dyn PlanBatchStore>,
    pub cache: RefCell<cache::TtlCache<PlanBatch>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PlanBatchStore for CachedPlanBatchStore {
    fn find(&self, code: &str) -> Option<PlanBatch> {
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

    fn save(&self, item: &PlanBatch) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PlanBatch> {
        self.inner.list()
    }
}

pub fn new_cached_plan_batch_store(
    inner: Rc<dyn PlanBatchStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PlanBatchStore> {
    Rc::new(CachedPlanBatchStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PlanBatch>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects plan batches. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanBatchQuery {
    pub title: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PlanBatchQuery {
    pub fn matches(&self, item: &PlanBatch) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PlanBatch]) -> Vec<PlanBatch> {
        let matched = collections::filter(items, |candidate: &PlanBatch| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_plan_batches(store: Rc<dyn PlanBatchStore>, query: &PlanBatchQuery) -> Vec<PlanBatch> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PlanBatchStore> = new_memory_plan_batch_store();
        let item = new_plan_batch("code-198", "title-776");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
