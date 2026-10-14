use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending plan items by code.
pub trait PendingPlanItemStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingPlanItem>;
    fn save(&self, item: &PendingPlanItem);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingPlanItem>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingPlanItemStore {
    pub items: RefCell<BTreeMap<String, PendingPlanItem>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingPlanItemStore for MemoryPendingPlanItemStore {
    fn find(&self, code: &str) -> Option<PendingPlanItem> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingPlanItem) {
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

    fn list(&self) -> Vec<PendingPlanItem> {
        let mut out: Vec<PendingPlanItem> = Vec::new();
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

pub fn new_memory_pending_plan_item_store() -> Rc<dyn PendingPlanItemStore> {
    Rc::new(MemoryPendingPlanItemStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingPlanItemStore {
    pub inner: Rc<dyn PendingPlanItemStore>,
    pub cache: RefCell<cache::TtlCache<PendingPlanItem>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingPlanItemStore for CachedPendingPlanItemStore {
    fn find(&self, code: &str) -> Option<PendingPlanItem> {
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

    fn save(&self, item: &PendingPlanItem) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingPlanItem> {
        self.inner.list()
    }
}

pub fn new_cached_pending_plan_item_store(
    inner: Rc<dyn PendingPlanItemStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingPlanItemStore> {
    Rc::new(CachedPendingPlanItemStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingPlanItem>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending plan items. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanItemQuery {
    pub scope: String,
    pub min_ends_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingPlanItemQuery {
    pub fn matches(&self, item: &PendingPlanItem) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
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

    pub fn apply(&self, items: &[PendingPlanItem]) -> Vec<PendingPlanItem> {
        let matched = collections::filter(items, |candidate: &PendingPlanItem| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_plan_items(
    store: Rc<dyn PendingPlanItemStore>,
    query: &PendingPlanItemQuery,
) -> Vec<PendingPlanItem> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PendingPlanItemStore> = new_memory_pending_plan_item_store();
        let item = new_pending_plan_item("code-695", "title-701");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
