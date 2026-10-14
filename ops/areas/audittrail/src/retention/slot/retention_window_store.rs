use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps retention windows by code.
pub trait RetentionWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RetentionWindow>;
    fn save(&self, item: &RetentionWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RetentionWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRetentionWindowStore {
    pub items: RefCell<BTreeMap<String, RetentionWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl RetentionWindowStore for MemoryRetentionWindowStore {
    fn find(&self, code: &str) -> Option<RetentionWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RetentionWindow) {
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

    fn list(&self) -> Vec<RetentionWindow> {
        let mut out: Vec<RetentionWindow> = Vec::new();
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

pub fn new_memory_retention_window_store() -> Rc<dyn RetentionWindowStore> {
    Rc::new(MemoryRetentionWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRetentionWindowStore {
    pub inner: Rc<dyn RetentionWindowStore>,
    pub cache: RefCell<cache::TtlCache<RetentionWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RetentionWindowStore for CachedRetentionWindowStore {
    fn find(&self, code: &str) -> Option<RetentionWindow> {
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

    fn save(&self, item: &RetentionWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RetentionWindow> {
        self.inner.list()
    }
}

pub fn new_cached_retention_window_store(
    inner: Rc<dyn RetentionWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RetentionWindowStore> {
    Rc::new(CachedRetentionWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RetentionWindow>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects retention windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionWindowQuery {
    pub cluster: String,
    pub min_batch_size: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RetentionWindowQuery {
    pub fn matches(&self, item: &RetentionWindow) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.batch_size < self.min_batch_size {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RetentionWindow]) -> Vec<RetentionWindow> {
        let matched = collections::filter(items, |candidate: &RetentionWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_retention_windows(
    store: Rc<dyn RetentionWindowStore>,
    query: &RetentionWindowQuery,
) -> Vec<RetentionWindow> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RetentionWindowStore> = new_memory_retention_window_store();
        let item = new_retention_window("code-867", "name-557");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
