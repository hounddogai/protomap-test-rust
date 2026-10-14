use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global segment items by code.
pub trait GlobalSegmentItemStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalSegmentItem>;
    fn save(&self, item: &GlobalSegmentItem);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalSegmentItem>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalSegmentItemStore {
    pub items: RefCell<BTreeMap<String, GlobalSegmentItem>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalSegmentItemStore for MemoryGlobalSegmentItemStore {
    fn find(&self, code: &str) -> Option<GlobalSegmentItem> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalSegmentItem) {
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

    fn list(&self) -> Vec<GlobalSegmentItem> {
        let mut out: Vec<GlobalSegmentItem> = Vec::new();
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

pub fn new_memory_global_segment_item_store() -> Rc<dyn GlobalSegmentItemStore> {
    Rc::new(MemoryGlobalSegmentItemStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalSegmentItemStore {
    pub inner: Rc<dyn GlobalSegmentItemStore>,
    pub cache: RefCell<cache::TtlCache<GlobalSegmentItem>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalSegmentItemStore for CachedGlobalSegmentItemStore {
    fn find(&self, code: &str) -> Option<GlobalSegmentItem> {
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

    fn save(&self, item: &GlobalSegmentItem) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalSegmentItem> {
        self.inner.list()
    }
}

pub fn new_cached_global_segment_item_store(
    inner: Rc<dyn GlobalSegmentItemStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalSegmentItemStore> {
    Rc::new(CachedGlobalSegmentItemStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalSegmentItem>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global segment items. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentItemQuery {
    pub name: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalSegmentItemQuery {
    pub fn matches(&self, item: &GlobalSegmentItem) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[GlobalSegmentItem]) -> Vec<GlobalSegmentItem> {
        let matched = collections::filter(items, |candidate: &GlobalSegmentItem| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_segment_items(
    store: Rc<dyn GlobalSegmentItemStore>,
    query: &GlobalSegmentItemQuery,
) -> Vec<GlobalSegmentItem> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalSegmentItemStore> = new_memory_global_segment_item_store();
        let item = new_global_segment_item("code-725", "name-186");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
