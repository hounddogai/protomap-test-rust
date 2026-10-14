use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps postmortem slots by code.
pub trait PostmortemSlotStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PostmortemSlot>;
    fn save(&self, item: &PostmortemSlot);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PostmortemSlot>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPostmortemSlotStore {
    pub items: RefCell<BTreeMap<String, PostmortemSlot>>,
    pub order: RefCell<Vec<String>>,
}

impl PostmortemSlotStore for MemoryPostmortemSlotStore {
    fn find(&self, code: &str) -> Option<PostmortemSlot> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PostmortemSlot) {
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

    fn list(&self) -> Vec<PostmortemSlot> {
        let mut out: Vec<PostmortemSlot> = Vec::new();
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

pub fn new_memory_postmortem_slot_store() -> Rc<dyn PostmortemSlotStore> {
    Rc::new(MemoryPostmortemSlotStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPostmortemSlotStore {
    pub inner: Rc<dyn PostmortemSlotStore>,
    pub cache: RefCell<cache::TtlCache<PostmortemSlot>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PostmortemSlotStore for CachedPostmortemSlotStore {
    fn find(&self, code: &str) -> Option<PostmortemSlot> {
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

    fn save(&self, item: &PostmortemSlot) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PostmortemSlot> {
        self.inner.list()
    }
}

pub fn new_cached_postmortem_slot_store(
    inner: Rc<dyn PostmortemSlotStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PostmortemSlotStore> {
    Rc::new(CachedPostmortemSlotStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PostmortemSlot>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects postmortem slots. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemSlotQuery {
    pub namespace: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PostmortemSlotQuery {
    pub fn matches(&self, item: &PostmortemSlot) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[PostmortemSlot]) -> Vec<PostmortemSlot> {
        let matched = collections::filter(items, |candidate: &PostmortemSlot| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_postmortem_slots(store: Rc<dyn PostmortemSlotStore>, query: &PostmortemSlotQuery) -> Vec<PostmortemSlot> {
    query.apply(&store.list())
}
