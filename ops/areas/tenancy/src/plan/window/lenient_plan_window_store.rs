use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient plan windows by code.
pub trait LenientPlanWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientPlanWindow>;
    fn save(&self, item: &LenientPlanWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientPlanWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientPlanWindowStore {
    pub items: RefCell<BTreeMap<String, LenientPlanWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientPlanWindowStore for MemoryLenientPlanWindowStore {
    fn find(&self, code: &str) -> Option<LenientPlanWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientPlanWindow) {
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

    fn list(&self) -> Vec<LenientPlanWindow> {
        let mut out: Vec<LenientPlanWindow> = Vec::new();
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

pub fn new_memory_lenient_plan_window_store() -> Rc<dyn LenientPlanWindowStore> {
    Rc::new(MemoryLenientPlanWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientPlanWindowStore {
    pub inner: Rc<dyn LenientPlanWindowStore>,
    pub cache: RefCell<cache::TtlCache<LenientPlanWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientPlanWindowStore for CachedLenientPlanWindowStore {
    fn find(&self, code: &str) -> Option<LenientPlanWindow> {
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

    fn save(&self, item: &LenientPlanWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientPlanWindow> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_plan_window_store(
    inner: Rc<dyn LenientPlanWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientPlanWindowStore> {
    Rc::new(CachedLenientPlanWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientPlanWindow>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient plan windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanWindowQuery {
    pub label: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientPlanWindowQuery {
    pub fn matches(&self, item: &LenientPlanWindow) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[LenientPlanWindow]) -> Vec<LenientPlanWindow> {
        let matched = collections::filter(items, |candidate: &LenientPlanWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_plan_windows(
    store: Rc<dyn LenientPlanWindowStore>,
    query: &LenientPlanWindowQuery,
) -> Vec<LenientPlanWindow> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientPlanWindowStore> = new_memory_lenient_plan_window_store();
        let item = new_lenient_plan_window("code-373", "label-467");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
