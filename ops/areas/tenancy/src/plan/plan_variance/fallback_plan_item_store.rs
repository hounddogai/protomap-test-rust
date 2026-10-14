use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps fallback plan items by code.
pub trait FallbackPlanItemStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<FallbackPlanItem>;
    fn save(&self, item: &FallbackPlanItem);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<FallbackPlanItem>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryFallbackPlanItemStore {
    pub items: RefCell<BTreeMap<String, FallbackPlanItem>>,
    pub order: RefCell<Vec<String>>,
}

impl FallbackPlanItemStore for MemoryFallbackPlanItemStore {
    fn find(&self, code: &str) -> Option<FallbackPlanItem> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &FallbackPlanItem) {
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

    fn list(&self) -> Vec<FallbackPlanItem> {
        let mut out: Vec<FallbackPlanItem> = Vec::new();
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

pub fn new_memory_fallback_plan_item_store() -> Rc<dyn FallbackPlanItemStore> {
    Rc::new(MemoryFallbackPlanItemStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedFallbackPlanItemStore {
    pub inner: Rc<dyn FallbackPlanItemStore>,
    pub cache: RefCell<cache::TtlCache<FallbackPlanItem>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl FallbackPlanItemStore for CachedFallbackPlanItemStore {
    fn find(&self, code: &str) -> Option<FallbackPlanItem> {
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

    fn save(&self, item: &FallbackPlanItem) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<FallbackPlanItem> {
        self.inner.list()
    }
}

pub fn new_cached_fallback_plan_item_store(
    inner: Rc<dyn FallbackPlanItemStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn FallbackPlanItemStore> {
    Rc::new(CachedFallbackPlanItemStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<FallbackPlanItem>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects fallback plan items. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanItemQuery {
    pub title: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl FallbackPlanItemQuery {
    pub fn matches(&self, item: &FallbackPlanItem) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[FallbackPlanItem]) -> Vec<FallbackPlanItem> {
        let matched = collections::filter(items, |candidate: &FallbackPlanItem| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_fallback_plan_items(
    store: Rc<dyn FallbackPlanItemStore>,
    query: &FallbackPlanItemQuery,
) -> Vec<FallbackPlanItem> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn FallbackPlanItemStore> = new_memory_fallback_plan_item_store();
        let item = new_fallback_plan_item("code-864", "title-7");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
