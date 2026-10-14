use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rule items by code.
pub trait RuleItemStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RuleItem>;
    fn save(&self, item: &RuleItem);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RuleItem>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRuleItemStore {
    pub items: RefCell<BTreeMap<String, RuleItem>>,
    pub order: RefCell<Vec<String>>,
}

impl RuleItemStore for MemoryRuleItemStore {
    fn find(&self, code: &str) -> Option<RuleItem> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RuleItem) {
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

    fn list(&self) -> Vec<RuleItem> {
        let mut out: Vec<RuleItem> = Vec::new();
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

pub fn new_memory_rule_item_store() -> Rc<dyn RuleItemStore> {
    Rc::new(MemoryRuleItemStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRuleItemStore {
    pub inner: Rc<dyn RuleItemStore>,
    pub cache: RefCell<cache::TtlCache<RuleItem>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RuleItemStore for CachedRuleItemStore {
    fn find(&self, code: &str) -> Option<RuleItem> {
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

    fn save(&self, item: &RuleItem) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RuleItem> {
        self.inner.list()
    }
}

pub fn new_cached_rule_item_store(
    inner: Rc<dyn RuleItemStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RuleItemStore> {
    Rc::new(CachedRuleItemStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RuleItem>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rule items. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleItemQuery {
    pub title: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RuleItemQuery {
    pub fn matches(&self, item: &RuleItem) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.cost < self.min_cost {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RuleItem]) -> Vec<RuleItem> {
        let matched = collections::filter(items, |candidate: &RuleItem| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rule_items(store: Rc<dyn RuleItemStore>, query: &RuleItemQuery) -> Vec<RuleItem> {
    query.apply(&store.list())
}
