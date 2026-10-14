use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps page tiers by code.
pub trait PageTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PageTier>;
    fn save(&self, item: &PageTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PageTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPageTierStore {
    pub items: RefCell<BTreeMap<String, PageTier>>,
    pub order: RefCell<Vec<String>>,
}

impl PageTierStore for MemoryPageTierStore {
    fn find(&self, code: &str) -> Option<PageTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PageTier) {
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

    fn list(&self) -> Vec<PageTier> {
        let mut out: Vec<PageTier> = Vec::new();
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

pub fn new_memory_page_tier_store() -> Rc<dyn PageTierStore> {
    Rc::new(MemoryPageTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPageTierStore {
    pub inner: Rc<dyn PageTierStore>,
    pub cache: RefCell<cache::TtlCache<PageTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PageTierStore for CachedPageTierStore {
    fn find(&self, code: &str) -> Option<PageTier> {
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

    fn save(&self, item: &PageTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PageTier> {
        self.inner.list()
    }
}

pub fn new_cached_page_tier_store(
    inner: Rc<dyn PageTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PageTierStore> {
    Rc::new(CachedPageTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PageTier>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects page tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PageTierQuery {
    pub name: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PageTierQuery {
    pub fn matches(&self, item: &PageTier) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[PageTier]) -> Vec<PageTier> {
        let matched = collections::filter(items, |candidate: &PageTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_page_tiers(store: Rc<dyn PageTierStore>, query: &PageTierQuery) -> Vec<PageTier> {
    query.apply(&store.list())
}
