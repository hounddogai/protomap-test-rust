use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps dashboard windows by code.
pub trait DashboardWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DashboardWindow>;
    fn save(&self, item: &DashboardWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DashboardWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDashboardWindowStore {
    pub items: RefCell<BTreeMap<String, DashboardWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl DashboardWindowStore for MemoryDashboardWindowStore {
    fn find(&self, code: &str) -> Option<DashboardWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DashboardWindow) {
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

    fn list(&self) -> Vec<DashboardWindow> {
        let mut out: Vec<DashboardWindow> = Vec::new();
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

pub fn new_memory_dashboard_window_store() -> Rc<dyn DashboardWindowStore> {
    Rc::new(MemoryDashboardWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDashboardWindowStore {
    pub inner: Rc<dyn DashboardWindowStore>,
    pub cache: RefCell<cache::TtlCache<DashboardWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DashboardWindowStore for CachedDashboardWindowStore {
    fn find(&self, code: &str) -> Option<DashboardWindow> {
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

    fn save(&self, item: &DashboardWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DashboardWindow> {
        self.inner.list()
    }
}

pub fn new_cached_dashboard_window_store(
    inner: Rc<dyn DashboardWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DashboardWindowStore> {
    Rc::new(CachedDashboardWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DashboardWindow>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects dashboard windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardWindowQuery {
    pub locale: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DashboardWindowQuery {
    pub fn matches(&self, item: &DashboardWindow) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.expires_at < self.min_expires_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DashboardWindow]) -> Vec<DashboardWindow> {
        let matched = collections::filter(items, |candidate: &DashboardWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_dashboard_windows(
    store: Rc<dyn DashboardWindowStore>,
    query: &DashboardWindowQuery,
) -> Vec<DashboardWindow> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DashboardWindowStore> = new_memory_dashboard_window_store();
        let item = new_dashboard_window("code-660", "name-660");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
