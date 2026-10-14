use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps limit cycles by code.
pub trait LimitCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LimitCycle>;
    fn save(&self, item: &LimitCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LimitCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLimitCycleStore {
    pub items: RefCell<BTreeMap<String, LimitCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl LimitCycleStore for MemoryLimitCycleStore {
    fn find(&self, code: &str) -> Option<LimitCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LimitCycle) {
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

    fn list(&self) -> Vec<LimitCycle> {
        let mut out: Vec<LimitCycle> = Vec::new();
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

pub fn new_memory_limit_cycle_store() -> Rc<dyn LimitCycleStore> {
    Rc::new(MemoryLimitCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLimitCycleStore {
    pub inner: Rc<dyn LimitCycleStore>,
    pub cache: RefCell<cache::TtlCache<LimitCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LimitCycleStore for CachedLimitCycleStore {
    fn find(&self, code: &str) -> Option<LimitCycle> {
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

    fn save(&self, item: &LimitCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LimitCycle> {
        self.inner.list()
    }
}

pub fn new_cached_limit_cycle_store(
    inner: Rc<dyn LimitCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LimitCycleStore> {
    Rc::new(CachedLimitCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LimitCycle>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects limit cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitCycleQuery {
    pub label: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LimitCycleQuery {
    pub fn matches(&self, item: &LimitCycle) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[LimitCycle]) -> Vec<LimitCycle> {
        let matched = collections::filter(items, |candidate: &LimitCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_limit_cycles(store: Rc<dyn LimitCycleStore>, query: &LimitCycleQuery) -> Vec<LimitCycle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LimitCycleStore> = new_memory_limit_cycle_store();
        let item = new_limit_cycle("code-356", "label-626");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
