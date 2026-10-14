use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps action item overrides by code.
pub trait ActionItemOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ActionItemOverride>;
    fn save(&self, item: &ActionItemOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ActionItemOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryActionItemOverrideStore {
    pub items: RefCell<BTreeMap<String, ActionItemOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl ActionItemOverrideStore for MemoryActionItemOverrideStore {
    fn find(&self, code: &str) -> Option<ActionItemOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ActionItemOverride) {
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

    fn list(&self) -> Vec<ActionItemOverride> {
        let mut out: Vec<ActionItemOverride> = Vec::new();
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

pub fn new_memory_action_item_override_store() -> Rc<dyn ActionItemOverrideStore> {
    Rc::new(MemoryActionItemOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedActionItemOverrideStore {
    pub inner: Rc<dyn ActionItemOverrideStore>,
    pub cache: RefCell<cache::TtlCache<ActionItemOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ActionItemOverrideStore for CachedActionItemOverrideStore {
    fn find(&self, code: &str) -> Option<ActionItemOverride> {
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

    fn save(&self, item: &ActionItemOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ActionItemOverride> {
        self.inner.list()
    }
}

pub fn new_cached_action_item_override_store(
    inner: Rc<dyn ActionItemOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ActionItemOverrideStore> {
    Rc::new(CachedActionItemOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ActionItemOverride>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects action item overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemOverrideQuery {
    pub name: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ActionItemOverrideQuery {
    pub fn matches(&self, item: &ActionItemOverride) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[ActionItemOverride]) -> Vec<ActionItemOverride> {
        let matched = collections::filter(items, |candidate: &ActionItemOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_action_item_overrides(
    store: Rc<dyn ActionItemOverrideStore>,
    query: &ActionItemOverrideQuery,
) -> Vec<ActionItemOverride> {
    query.apply(&store.list())
}
