use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rule allocations by code.
pub trait RuleAllocationStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RuleAllocation>;
    fn save(&self, item: &RuleAllocation);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RuleAllocation>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRuleAllocationStore {
    pub items: RefCell<BTreeMap<String, RuleAllocation>>,
    pub order: RefCell<Vec<String>>,
}

impl RuleAllocationStore for MemoryRuleAllocationStore {
    fn find(&self, code: &str) -> Option<RuleAllocation> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RuleAllocation) {
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

    fn list(&self) -> Vec<RuleAllocation> {
        let mut out: Vec<RuleAllocation> = Vec::new();
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

pub fn new_memory_rule_allocation_store() -> Rc<dyn RuleAllocationStore> {
    Rc::new(MemoryRuleAllocationStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRuleAllocationStore {
    pub inner: Rc<dyn RuleAllocationStore>,
    pub cache: RefCell<cache::TtlCache<RuleAllocation>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RuleAllocationStore for CachedRuleAllocationStore {
    fn find(&self, code: &str) -> Option<RuleAllocation> {
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

    fn save(&self, item: &RuleAllocation) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RuleAllocation> {
        self.inner.list()
    }
}

pub fn new_cached_rule_allocation_store(
    inner: Rc<dyn RuleAllocationStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RuleAllocationStore> {
    Rc::new(CachedRuleAllocationStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RuleAllocation>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rule allocations. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleAllocationQuery {
    pub title: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RuleAllocationQuery {
    pub fn matches(&self, item: &RuleAllocation) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RuleAllocation]) -> Vec<RuleAllocation> {
        let matched = collections::filter(items, |candidate: &RuleAllocation| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rule_allocations(store: Rc<dyn RuleAllocationStore>, query: &RuleAllocationQuery) -> Vec<RuleAllocation> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RuleAllocationStore> = new_memory_rule_allocation_store();
        let item = new_rule_allocation("code-468", "title-2");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
