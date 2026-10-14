use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rollback baselines by code.
pub trait RollbackBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollbackBaseline>;
    fn save(&self, item: &RollbackBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollbackBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollbackBaselineStore {
    pub items: RefCell<BTreeMap<String, RollbackBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl RollbackBaselineStore for MemoryRollbackBaselineStore {
    fn find(&self, code: &str) -> Option<RollbackBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollbackBaseline) {
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

    fn list(&self) -> Vec<RollbackBaseline> {
        let mut out: Vec<RollbackBaseline> = Vec::new();
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

pub fn new_memory_rollback_baseline_store() -> Rc<dyn RollbackBaselineStore> {
    Rc::new(MemoryRollbackBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollbackBaselineStore {
    pub inner: Rc<dyn RollbackBaselineStore>,
    pub cache: RefCell<cache::TtlCache<RollbackBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollbackBaselineStore for CachedRollbackBaselineStore {
    fn find(&self, code: &str) -> Option<RollbackBaseline> {
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

    fn save(&self, item: &RollbackBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollbackBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_rollback_baseline_store(
    inner: Rc<dyn RollbackBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollbackBaselineStore> {
    Rc::new(CachedRollbackBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollbackBaseline>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rollback baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackBaselineQuery {
    pub segment: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollbackBaselineQuery {
    pub fn matches(&self, item: &RollbackBaseline) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollbackBaseline]) -> Vec<RollbackBaseline> {
        let matched = collections::filter(items, |candidate: &RollbackBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rollback_baselines(
    store: Rc<dyn RollbackBaselineStore>,
    query: &RollbackBaselineQuery,
) -> Vec<RollbackBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollbackBaselineStore> = new_memory_rollback_baseline_store();
        let item = new_rollback_baseline("code-964", "title-220");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
