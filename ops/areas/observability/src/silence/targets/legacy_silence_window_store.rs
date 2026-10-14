use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy silence windows by code.
pub trait LegacySilenceWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacySilenceWindow>;
    fn save(&self, item: &LegacySilenceWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacySilenceWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacySilenceWindowStore {
    pub items: RefCell<BTreeMap<String, LegacySilenceWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacySilenceWindowStore for MemoryLegacySilenceWindowStore {
    fn find(&self, code: &str) -> Option<LegacySilenceWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacySilenceWindow) {
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

    fn list(&self) -> Vec<LegacySilenceWindow> {
        let mut out: Vec<LegacySilenceWindow> = Vec::new();
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

pub fn new_memory_legacy_silence_window_store() -> Rc<dyn LegacySilenceWindowStore> {
    Rc::new(MemoryLegacySilenceWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacySilenceWindowStore {
    pub inner: Rc<dyn LegacySilenceWindowStore>,
    pub cache: RefCell<cache::TtlCache<LegacySilenceWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacySilenceWindowStore for CachedLegacySilenceWindowStore {
    fn find(&self, code: &str) -> Option<LegacySilenceWindow> {
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

    fn save(&self, item: &LegacySilenceWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacySilenceWindow> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_silence_window_store(
    inner: Rc<dyn LegacySilenceWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacySilenceWindowStore> {
    Rc::new(CachedLegacySilenceWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacySilenceWindow>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy silence windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceWindowQuery {
    pub cluster: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacySilenceWindowQuery {
    pub fn matches(&self, item: &LegacySilenceWindow) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[LegacySilenceWindow]) -> Vec<LegacySilenceWindow> {
        let matched = collections::filter(items, |candidate: &LegacySilenceWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_silence_windows(
    store: Rc<dyn LegacySilenceWindowStore>,
    query: &LegacySilenceWindowQuery,
) -> Vec<LegacySilenceWindow> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacySilenceWindowStore> = new_memory_legacy_silence_window_store();
        let item = new_legacy_silence_window("code-572", "title-648");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
