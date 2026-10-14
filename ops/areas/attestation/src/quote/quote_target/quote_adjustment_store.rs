use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps quote adjustments by code.
pub trait QuoteAdjustmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<QuoteAdjustment>;
    fn save(&self, item: &QuoteAdjustment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<QuoteAdjustment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryQuoteAdjustmentStore {
    pub items: RefCell<BTreeMap<String, QuoteAdjustment>>,
    pub order: RefCell<Vec<String>>,
}

impl QuoteAdjustmentStore for MemoryQuoteAdjustmentStore {
    fn find(&self, code: &str) -> Option<QuoteAdjustment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &QuoteAdjustment) {
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

    fn list(&self) -> Vec<QuoteAdjustment> {
        let mut out: Vec<QuoteAdjustment> = Vec::new();
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

pub fn new_memory_quote_adjustment_store() -> Rc<dyn QuoteAdjustmentStore> {
    Rc::new(MemoryQuoteAdjustmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedQuoteAdjustmentStore {
    pub inner: Rc<dyn QuoteAdjustmentStore>,
    pub cache: RefCell<cache::TtlCache<QuoteAdjustment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl QuoteAdjustmentStore for CachedQuoteAdjustmentStore {
    fn find(&self, code: &str) -> Option<QuoteAdjustment> {
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

    fn save(&self, item: &QuoteAdjustment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<QuoteAdjustment> {
        self.inner.list()
    }
}

pub fn new_cached_quote_adjustment_store(
    inner: Rc<dyn QuoteAdjustmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn QuoteAdjustmentStore> {
    Rc::new(CachedQuoteAdjustmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<QuoteAdjustment>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects quote adjustments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAdjustmentQuery {
    pub namespace: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl QuoteAdjustmentQuery {
    pub fn matches(&self, item: &QuoteAdjustment) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[QuoteAdjustment]) -> Vec<QuoteAdjustment> {
        let matched = collections::filter(items, |candidate: &QuoteAdjustment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_quote_adjustments(
    store: Rc<dyn QuoteAdjustmentStore>,
    query: &QuoteAdjustmentQuery,
) -> Vec<QuoteAdjustment> {
    query.apply(&store.list())
}
