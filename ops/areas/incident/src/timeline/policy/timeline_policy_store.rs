use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps timeline policies by code.
pub trait TimelinePolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TimelinePolicy>;
    fn save(&self, item: &TimelinePolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TimelinePolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTimelinePolicyStore {
    pub items: RefCell<BTreeMap<String, TimelinePolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl TimelinePolicyStore for MemoryTimelinePolicyStore {
    fn find(&self, code: &str) -> Option<TimelinePolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TimelinePolicy) {
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

    fn list(&self) -> Vec<TimelinePolicy> {
        let mut out: Vec<TimelinePolicy> = Vec::new();
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

pub fn new_memory_timeline_policy_store() -> Rc<dyn TimelinePolicyStore> {
    Rc::new(MemoryTimelinePolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTimelinePolicyStore {
    pub inner: Rc<dyn TimelinePolicyStore>,
    pub cache: RefCell<cache::TtlCache<TimelinePolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TimelinePolicyStore for CachedTimelinePolicyStore {
    fn find(&self, code: &str) -> Option<TimelinePolicy> {
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

    fn save(&self, item: &TimelinePolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TimelinePolicy> {
        self.inner.list()
    }
}

pub fn new_cached_timeline_policy_store(
    inner: Rc<dyn TimelinePolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TimelinePolicyStore> {
    Rc::new(CachedTimelinePolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TimelinePolicy>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects timeline policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePolicyQuery {
    pub cluster: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TimelinePolicyQuery {
    pub fn matches(&self, item: &TimelinePolicy) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[TimelinePolicy]) -> Vec<TimelinePolicy> {
        let matched = collections::filter(items, |candidate: &TimelinePolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_timeline_policies(store: Rc<dyn TimelinePolicyStore>, query: &TimelinePolicyQuery) -> Vec<TimelinePolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TimelinePolicyStore> = new_memory_timeline_policy_store();
        let item = new_timeline_policy("code-338", "label-410");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
