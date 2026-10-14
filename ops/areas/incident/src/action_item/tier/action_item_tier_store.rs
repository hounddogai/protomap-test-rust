use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps action item tiers by code.
pub trait ActionItemTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ActionItemTier>;
    fn save(&self, item: &ActionItemTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ActionItemTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryActionItemTierStore {
    pub items: RefCell<BTreeMap<String, ActionItemTier>>,
    pub order: RefCell<Vec<String>>,
}

impl ActionItemTierStore for MemoryActionItemTierStore {
    fn find(&self, code: &str) -> Option<ActionItemTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ActionItemTier) {
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

    fn list(&self) -> Vec<ActionItemTier> {
        let mut out: Vec<ActionItemTier> = Vec::new();
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

pub fn new_memory_action_item_tier_store() -> Rc<dyn ActionItemTierStore> {
    Rc::new(MemoryActionItemTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedActionItemTierStore {
    pub inner: Rc<dyn ActionItemTierStore>,
    pub cache: RefCell<cache::TtlCache<ActionItemTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ActionItemTierStore for CachedActionItemTierStore {
    fn find(&self, code: &str) -> Option<ActionItemTier> {
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

    fn save(&self, item: &ActionItemTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ActionItemTier> {
        self.inner.list()
    }
}

pub fn new_cached_action_item_tier_store(
    inner: Rc<dyn ActionItemTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ActionItemTierStore> {
    Rc::new(CachedActionItemTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ActionItemTier>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects action item tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTierQuery {
    pub label: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ActionItemTierQuery {
    pub fn matches(&self, item: &ActionItemTier) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ActionItemTier]) -> Vec<ActionItemTier> {
        let matched = collections::filter(items, |candidate: &ActionItemTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_action_item_tiers(store: Rc<dyn ActionItemTierStore>, query: &ActionItemTierQuery) -> Vec<ActionItemTier> {
    query.apply(&store.list())
}
