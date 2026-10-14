use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rollback windows by code.
pub trait RollbackWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollbackWindow>;
    fn save(&self, item: &RollbackWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollbackWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollbackWindowStore {
    pub items: RefCell<BTreeMap<String, RollbackWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl RollbackWindowStore for MemoryRollbackWindowStore {
    fn find(&self, code: &str) -> Option<RollbackWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollbackWindow) {
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

    fn list(&self) -> Vec<RollbackWindow> {
        let mut out: Vec<RollbackWindow> = Vec::new();
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

pub fn new_memory_rollback_window_store() -> Rc<dyn RollbackWindowStore> {
    Rc::new(MemoryRollbackWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollbackWindowStore {
    pub inner: Rc<dyn RollbackWindowStore>,
    pub cache: RefCell<cache::TtlCache<RollbackWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollbackWindowStore for CachedRollbackWindowStore {
    fn find(&self, code: &str) -> Option<RollbackWindow> {
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

    fn save(&self, item: &RollbackWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollbackWindow> {
        self.inner.list()
    }
}

pub fn new_cached_rollback_window_store(
    inner: Rc<dyn RollbackWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollbackWindowStore> {
    Rc::new(CachedRollbackWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollbackWindow>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rollback windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackWindowQuery {
    pub scope: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollbackWindowQuery {
    pub fn matches(&self, item: &RollbackWindow) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollbackWindow]) -> Vec<RollbackWindow> {
        let matched = collections::filter(items, |candidate: &RollbackWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rollback_windows(store: Rc<dyn RollbackWindowStore>, query: &RollbackWindowQuery) -> Vec<RollbackWindow> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollbackWindowStore> = new_memory_rollback_window_store();
        let item = new_rollback_window("code-426", "label-614");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
