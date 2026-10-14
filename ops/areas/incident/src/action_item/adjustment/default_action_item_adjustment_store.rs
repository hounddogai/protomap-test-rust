use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default action item adjustments by code.
pub trait DefaultActionItemAdjustmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultActionItemAdjustment>;
    fn save(&self, item: &DefaultActionItemAdjustment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultActionItemAdjustment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultActionItemAdjustmentStore {
    pub items: RefCell<BTreeMap<String, DefaultActionItemAdjustment>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultActionItemAdjustmentStore for MemoryDefaultActionItemAdjustmentStore {
    fn find(&self, code: &str) -> Option<DefaultActionItemAdjustment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultActionItemAdjustment) {
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

    fn list(&self) -> Vec<DefaultActionItemAdjustment> {
        let mut out: Vec<DefaultActionItemAdjustment> = Vec::new();
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

pub fn new_memory_default_action_item_adjustment_store() -> Rc<dyn DefaultActionItemAdjustmentStore> {
    Rc::new(MemoryDefaultActionItemAdjustmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultActionItemAdjustmentStore {
    pub inner: Rc<dyn DefaultActionItemAdjustmentStore>,
    pub cache: RefCell<cache::TtlCache<DefaultActionItemAdjustment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultActionItemAdjustmentStore for CachedDefaultActionItemAdjustmentStore {
    fn find(&self, code: &str) -> Option<DefaultActionItemAdjustment> {
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

    fn save(&self, item: &DefaultActionItemAdjustment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultActionItemAdjustment> {
        self.inner.list()
    }
}

pub fn new_cached_default_action_item_adjustment_store(
    inner: Rc<dyn DefaultActionItemAdjustmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultActionItemAdjustmentStore> {
    Rc::new(CachedDefaultActionItemAdjustmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultActionItemAdjustment>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default action item adjustments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemAdjustmentQuery {
    pub scope: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultActionItemAdjustmentQuery {
    pub fn matches(&self, item: &DefaultActionItemAdjustment) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultActionItemAdjustment]) -> Vec<DefaultActionItemAdjustment> {
        let matched = collections::filter(items, |candidate: &DefaultActionItemAdjustment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_action_item_adjustments(
    store: Rc<dyn DefaultActionItemAdjustmentStore>,
    query: &DefaultActionItemAdjustmentQuery,
) -> Vec<DefaultActionItemAdjustment> {
    query.apply(&store.list())
}
