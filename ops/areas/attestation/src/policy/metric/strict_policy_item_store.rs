use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict policy items by code.
pub trait StrictPolicyItemStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictPolicyItem>;
    fn save(&self, item: &StrictPolicyItem);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictPolicyItem>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictPolicyItemStore {
    pub items: RefCell<BTreeMap<String, StrictPolicyItem>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictPolicyItemStore for MemoryStrictPolicyItemStore {
    fn find(&self, code: &str) -> Option<StrictPolicyItem> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictPolicyItem) {
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

    fn list(&self) -> Vec<StrictPolicyItem> {
        let mut out: Vec<StrictPolicyItem> = Vec::new();
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

pub fn new_memory_strict_policy_item_store() -> Rc<dyn StrictPolicyItemStore> {
    Rc::new(MemoryStrictPolicyItemStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictPolicyItemStore {
    pub inner: Rc<dyn StrictPolicyItemStore>,
    pub cache: RefCell<cache::TtlCache<StrictPolicyItem>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictPolicyItemStore for CachedStrictPolicyItemStore {
    fn find(&self, code: &str) -> Option<StrictPolicyItem> {
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

    fn save(&self, item: &StrictPolicyItem) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictPolicyItem> {
        self.inner.list()
    }
}

pub fn new_cached_strict_policy_item_store(
    inner: Rc<dyn StrictPolicyItemStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictPolicyItemStore> {
    Rc::new(CachedStrictPolicyItemStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictPolicyItem>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict policy items. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyItemQuery {
    pub source: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictPolicyItemQuery {
    pub fn matches(&self, item: &StrictPolicyItem) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictPolicyItem]) -> Vec<StrictPolicyItem> {
        let matched = collections::filter(items, |candidate: &StrictPolicyItem| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_policy_items(
    store: Rc<dyn StrictPolicyItemStore>,
    query: &StrictPolicyItemQuery,
) -> Vec<StrictPolicyItem> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn StrictPolicyItemStore> = new_memory_strict_policy_item_store();
        let item = new_strict_policy_item("code-816", "name-66");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
