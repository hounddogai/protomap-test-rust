use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default query adjustments by code.
pub trait DefaultQueryAdjustmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultQueryAdjustment>;
    fn save(&self, item: &DefaultQueryAdjustment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultQueryAdjustment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultQueryAdjustmentStore {
    pub items: RefCell<BTreeMap<String, DefaultQueryAdjustment>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultQueryAdjustmentStore for MemoryDefaultQueryAdjustmentStore {
    fn find(&self, code: &str) -> Option<DefaultQueryAdjustment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultQueryAdjustment) {
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

    fn list(&self) -> Vec<DefaultQueryAdjustment> {
        let mut out: Vec<DefaultQueryAdjustment> = Vec::new();
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

pub fn new_memory_default_query_adjustment_store() -> Rc<dyn DefaultQueryAdjustmentStore> {
    Rc::new(MemoryDefaultQueryAdjustmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultQueryAdjustmentStore {
    pub inner: Rc<dyn DefaultQueryAdjustmentStore>,
    pub cache: RefCell<cache::TtlCache<DefaultQueryAdjustment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultQueryAdjustmentStore for CachedDefaultQueryAdjustmentStore {
    fn find(&self, code: &str) -> Option<DefaultQueryAdjustment> {
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

    fn save(&self, item: &DefaultQueryAdjustment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultQueryAdjustment> {
        self.inner.list()
    }
}

pub fn new_cached_default_query_adjustment_store(
    inner: Rc<dyn DefaultQueryAdjustmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultQueryAdjustmentStore> {
    Rc::new(CachedDefaultQueryAdjustmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultQueryAdjustment>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default query adjustments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryAdjustmentQuery {
    pub channel: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultQueryAdjustmentQuery {
    pub fn matches(&self, item: &DefaultQueryAdjustment) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
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

    pub fn apply(&self, items: &[DefaultQueryAdjustment]) -> Vec<DefaultQueryAdjustment> {
        let matched = collections::filter(items, |candidate: &DefaultQueryAdjustment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_query_adjustments(
    store: Rc<dyn DefaultQueryAdjustmentStore>,
    query: &DefaultQueryAdjustmentQuery,
) -> Vec<DefaultQueryAdjustment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DefaultQueryAdjustmentStore> = new_memory_default_query_adjustment_store();
        let item = new_default_query_adjustment("code-588", "label-357");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
