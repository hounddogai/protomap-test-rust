use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps slo bundles by code.
pub trait SloBundleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SloBundle>;
    fn save(&self, item: &SloBundle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SloBundle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySloBundleStore {
    pub items: RefCell<BTreeMap<String, SloBundle>>,
    pub order: RefCell<Vec<String>>,
}

impl SloBundleStore for MemorySloBundleStore {
    fn find(&self, code: &str) -> Option<SloBundle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SloBundle) {
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

    fn list(&self) -> Vec<SloBundle> {
        let mut out: Vec<SloBundle> = Vec::new();
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

pub fn new_memory_slo_bundle_store() -> Rc<dyn SloBundleStore> {
    Rc::new(MemorySloBundleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSloBundleStore {
    pub inner: Rc<dyn SloBundleStore>,
    pub cache: RefCell<cache::TtlCache<SloBundle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SloBundleStore for CachedSloBundleStore {
    fn find(&self, code: &str) -> Option<SloBundle> {
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

    fn save(&self, item: &SloBundle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SloBundle> {
        self.inner.list()
    }
}

pub fn new_cached_slo_bundle_store(
    inner: Rc<dyn SloBundleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SloBundleStore> {
    Rc::new(CachedSloBundleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SloBundle>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects slo bundles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SloBundleQuery {
    pub label: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SloBundleQuery {
    pub fn matches(&self, item: &SloBundle) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.ceiling < self.min_ceiling {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SloBundle]) -> Vec<SloBundle> {
        let matched = collections::filter(items, |candidate: &SloBundle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_slo_bundles(store: Rc<dyn SloBundleStore>, query: &SloBundleQuery) -> Vec<SloBundle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SloBundleStore> = new_memory_slo_bundle_store();
        let item = new_slo_bundle("code-450", "label-376");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
