use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global dashboard notices by code.
pub trait GlobalDashboardNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalDashboardNotice>;
    fn save(&self, item: &GlobalDashboardNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalDashboardNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalDashboardNoticeStore {
    pub items: RefCell<BTreeMap<String, GlobalDashboardNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalDashboardNoticeStore for MemoryGlobalDashboardNoticeStore {
    fn find(&self, code: &str) -> Option<GlobalDashboardNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalDashboardNotice) {
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

    fn list(&self) -> Vec<GlobalDashboardNotice> {
        let mut out: Vec<GlobalDashboardNotice> = Vec::new();
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

pub fn new_memory_global_dashboard_notice_store() -> Rc<dyn GlobalDashboardNoticeStore> {
    Rc::new(MemoryGlobalDashboardNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalDashboardNoticeStore {
    pub inner: Rc<dyn GlobalDashboardNoticeStore>,
    pub cache: RefCell<cache::TtlCache<GlobalDashboardNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalDashboardNoticeStore for CachedGlobalDashboardNoticeStore {
    fn find(&self, code: &str) -> Option<GlobalDashboardNotice> {
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

    fn save(&self, item: &GlobalDashboardNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalDashboardNotice> {
        self.inner.list()
    }
}

pub fn new_cached_global_dashboard_notice_store(
    inner: Rc<dyn GlobalDashboardNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalDashboardNoticeStore> {
    Rc::new(CachedGlobalDashboardNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalDashboardNotice>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global dashboard notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDashboardNoticeQuery {
    pub label: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalDashboardNoticeQuery {
    pub fn matches(&self, item: &GlobalDashboardNotice) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalDashboardNotice]) -> Vec<GlobalDashboardNotice> {
        let matched = collections::filter(items, |candidate: &GlobalDashboardNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_dashboard_notices(
    store: Rc<dyn GlobalDashboardNoticeStore>,
    query: &GlobalDashboardNoticeQuery,
) -> Vec<GlobalDashboardNotice> {
    query.apply(&store.list())
}
