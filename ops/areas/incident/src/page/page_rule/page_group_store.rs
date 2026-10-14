use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps page groups by code.
pub trait PageGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PageGroup>;
    fn save(&self, item: &PageGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PageGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPageGroupStore {
    pub items: RefCell<BTreeMap<String, PageGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl PageGroupStore for MemoryPageGroupStore {
    fn find(&self, code: &str) -> Option<PageGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PageGroup) {
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

    fn list(&self) -> Vec<PageGroup> {
        let mut out: Vec<PageGroup> = Vec::new();
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

pub fn new_memory_page_group_store() -> Rc<dyn PageGroupStore> {
    Rc::new(MemoryPageGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPageGroupStore {
    pub inner: Rc<dyn PageGroupStore>,
    pub cache: RefCell<cache::TtlCache<PageGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PageGroupStore for CachedPageGroupStore {
    fn find(&self, code: &str) -> Option<PageGroup> {
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

    fn save(&self, item: &PageGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PageGroup> {
        self.inner.list()
    }
}

pub fn new_cached_page_group_store(
    inner: Rc<dyn PageGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PageGroupStore> {
    Rc::new(CachedPageGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PageGroup>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects page groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PageGroupQuery {
    pub name: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PageGroupQuery {
    pub fn matches(&self, item: &PageGroup) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PageGroup]) -> Vec<PageGroup> {
        let matched = collections::filter(items, |candidate: &PageGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_page_groups(store: Rc<dyn PageGroupStore>, query: &PageGroupQuery) -> Vec<PageGroup> {
    query.apply(&store.list())
}
