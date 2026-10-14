use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps slo windows by code.
pub trait SloWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SloWindow>;
    fn save(&self, item: &SloWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SloWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySloWindowStore {
    pub items: RefCell<BTreeMap<String, SloWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl SloWindowStore for MemorySloWindowStore {
    fn find(&self, code: &str) -> Option<SloWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SloWindow) {
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

    fn list(&self) -> Vec<SloWindow> {
        let mut out: Vec<SloWindow> = Vec::new();
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

pub fn new_memory_slo_window_store() -> Rc<dyn SloWindowStore> {
    Rc::new(MemorySloWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSloWindowStore {
    pub inner: Rc<dyn SloWindowStore>,
    pub cache: RefCell<cache::TtlCache<SloWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SloWindowStore for CachedSloWindowStore {
    fn find(&self, code: &str) -> Option<SloWindow> {
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

    fn save(&self, item: &SloWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SloWindow> {
        self.inner.list()
    }
}

pub fn new_cached_slo_window_store(
    inner: Rc<dyn SloWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SloWindowStore> {
    Rc::new(CachedSloWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SloWindow>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects slo windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SloWindowQuery {
    pub name: String,
    pub min_floor_value: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SloWindowQuery {
    pub fn matches(&self, item: &SloWindow) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.floor_value < self.min_floor_value {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SloWindow]) -> Vec<SloWindow> {
        let matched = collections::filter(items, |candidate: &SloWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_slo_windows(store: Rc<dyn SloWindowStore>, query: &SloWindowQuery) -> Vec<SloWindow> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SloWindowStore> = new_memory_slo_window_store();
        let item = new_slo_window("code-982", "name-491");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
