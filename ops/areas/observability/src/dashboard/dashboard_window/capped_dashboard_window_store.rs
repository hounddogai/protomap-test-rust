use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped dashboard windows by code.
pub trait CappedDashboardWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedDashboardWindow>;
    fn save(&self, item: &CappedDashboardWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedDashboardWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedDashboardWindowStore {
    pub items: RefCell<BTreeMap<String, CappedDashboardWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedDashboardWindowStore for MemoryCappedDashboardWindowStore {
    fn find(&self, code: &str) -> Option<CappedDashboardWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedDashboardWindow) {
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

    fn list(&self) -> Vec<CappedDashboardWindow> {
        let mut out: Vec<CappedDashboardWindow> = Vec::new();
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

pub fn new_memory_capped_dashboard_window_store() -> Rc<dyn CappedDashboardWindowStore> {
    Rc::new(MemoryCappedDashboardWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedDashboardWindowStore {
    pub inner: Rc<dyn CappedDashboardWindowStore>,
    pub cache: RefCell<cache::TtlCache<CappedDashboardWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedDashboardWindowStore for CachedCappedDashboardWindowStore {
    fn find(&self, code: &str) -> Option<CappedDashboardWindow> {
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

    fn save(&self, item: &CappedDashboardWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedDashboardWindow> {
        self.inner.list()
    }
}

pub fn new_cached_capped_dashboard_window_store(
    inner: Rc<dyn CappedDashboardWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedDashboardWindowStore> {
    Rc::new(CachedCappedDashboardWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedDashboardWindow>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped dashboard windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardWindowQuery {
    pub namespace: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedDashboardWindowQuery {
    pub fn matches(&self, item: &CappedDashboardWindow) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.ends_at < self.min_ends_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedDashboardWindow]) -> Vec<CappedDashboardWindow> {
        let matched = collections::filter(items, |candidate: &CappedDashboardWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_dashboard_windows(
    store: Rc<dyn CappedDashboardWindowStore>,
    query: &CappedDashboardWindowQuery,
) -> Vec<CappedDashboardWindow> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedDashboardWindowStore> = new_memory_capped_dashboard_window_store();
        let item = new_capped_dashboard_window("code-862", "title-503");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
