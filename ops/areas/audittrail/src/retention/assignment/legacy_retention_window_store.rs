use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy retention windows by code.
pub trait LegacyRetentionWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyRetentionWindow>;
    fn save(&self, item: &LegacyRetentionWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyRetentionWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyRetentionWindowStore {
    pub items: RefCell<BTreeMap<String, LegacyRetentionWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyRetentionWindowStore for MemoryLegacyRetentionWindowStore {
    fn find(&self, code: &str) -> Option<LegacyRetentionWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyRetentionWindow) {
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

    fn list(&self) -> Vec<LegacyRetentionWindow> {
        let mut out: Vec<LegacyRetentionWindow> = Vec::new();
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

pub fn new_memory_legacy_retention_window_store() -> Rc<dyn LegacyRetentionWindowStore> {
    Rc::new(MemoryLegacyRetentionWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyRetentionWindowStore {
    pub inner: Rc<dyn LegacyRetentionWindowStore>,
    pub cache: RefCell<cache::TtlCache<LegacyRetentionWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyRetentionWindowStore for CachedLegacyRetentionWindowStore {
    fn find(&self, code: &str) -> Option<LegacyRetentionWindow> {
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

    fn save(&self, item: &LegacyRetentionWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyRetentionWindow> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_retention_window_store(
    inner: Rc<dyn LegacyRetentionWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyRetentionWindowStore> {
    Rc::new(CachedLegacyRetentionWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyRetentionWindow>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy retention windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRetentionWindowQuery {
    pub segment: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyRetentionWindowQuery {
    pub fn matches(&self, item: &LegacyRetentionWindow) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyRetentionWindow]) -> Vec<LegacyRetentionWindow> {
        let matched = collections::filter(items, |candidate: &LegacyRetentionWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_retention_windows(
    store: Rc<dyn LegacyRetentionWindowStore>,
    query: &LegacyRetentionWindowQuery,
) -> Vec<LegacyRetentionWindow> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyRetentionWindowStore> = new_memory_legacy_retention_window_store();
        let item = new_legacy_retention_window("code-397", "label-968");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
