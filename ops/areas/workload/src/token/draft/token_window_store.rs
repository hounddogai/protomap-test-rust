use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps token windows by code.
pub trait TokenWindowStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TokenWindow>;
    fn save(&self, item: &TokenWindow);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TokenWindow>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTokenWindowStore {
    pub items: RefCell<BTreeMap<String, TokenWindow>>,
    pub order: RefCell<Vec<String>>,
}

impl TokenWindowStore for MemoryTokenWindowStore {
    fn find(&self, code: &str) -> Option<TokenWindow> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TokenWindow) {
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

    fn list(&self) -> Vec<TokenWindow> {
        let mut out: Vec<TokenWindow> = Vec::new();
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

pub fn new_memory_token_window_store() -> Rc<dyn TokenWindowStore> {
    Rc::new(MemoryTokenWindowStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTokenWindowStore {
    pub inner: Rc<dyn TokenWindowStore>,
    pub cache: RefCell<cache::TtlCache<TokenWindow>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TokenWindowStore for CachedTokenWindowStore {
    fn find(&self, code: &str) -> Option<TokenWindow> {
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

    fn save(&self, item: &TokenWindow) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TokenWindow> {
        self.inner.list()
    }
}

pub fn new_cached_token_window_store(
    inner: Rc<dyn TokenWindowStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TokenWindowStore> {
    Rc::new(CachedTokenWindowStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TokenWindow>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects token windows. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenWindowQuery {
    pub locale: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TokenWindowQuery {
    pub fn matches(&self, item: &TokenWindow) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TokenWindow]) -> Vec<TokenWindow> {
        let matched = collections::filter(items, |candidate: &TokenWindow| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_token_windows(store: Rc<dyn TokenWindowStore>, query: &TokenWindowQuery) -> Vec<TokenWindow> {
    query.apply(&store.list())
}
