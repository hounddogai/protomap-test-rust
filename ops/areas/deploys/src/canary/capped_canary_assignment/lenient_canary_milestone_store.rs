use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient canary milestones by code.
pub trait LenientCanaryMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientCanaryMilestone>;
    fn save(&self, item: &LenientCanaryMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientCanaryMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientCanaryMilestoneStore {
    pub items: RefCell<BTreeMap<String, LenientCanaryMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientCanaryMilestoneStore for MemoryLenientCanaryMilestoneStore {
    fn find(&self, code: &str) -> Option<LenientCanaryMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientCanaryMilestone) {
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

    fn list(&self) -> Vec<LenientCanaryMilestone> {
        let mut out: Vec<LenientCanaryMilestone> = Vec::new();
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

pub fn new_memory_lenient_canary_milestone_store() -> Rc<dyn LenientCanaryMilestoneStore> {
    Rc::new(MemoryLenientCanaryMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientCanaryMilestoneStore {
    pub inner: Rc<dyn LenientCanaryMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<LenientCanaryMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientCanaryMilestoneStore for CachedLenientCanaryMilestoneStore {
    fn find(&self, code: &str) -> Option<LenientCanaryMilestone> {
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

    fn save(&self, item: &LenientCanaryMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientCanaryMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_canary_milestone_store(
    inner: Rc<dyn LenientCanaryMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientCanaryMilestoneStore> {
    Rc::new(CachedLenientCanaryMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientCanaryMilestone>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient canary milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryMilestoneQuery {
    pub cluster: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientCanaryMilestoneQuery {
    pub fn matches(&self, item: &LenientCanaryMilestone) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.attempts < self.min_attempts {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientCanaryMilestone]) -> Vec<LenientCanaryMilestone> {
        let matched = collections::filter(items, |candidate: &LenientCanaryMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_canary_milestones(
    store: Rc<dyn LenientCanaryMilestoneStore>,
    query: &LenientCanaryMilestoneQuery,
) -> Vec<LenientCanaryMilestone> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientCanaryMilestoneStore> = new_memory_lenient_canary_milestone_store();
        let item = new_lenient_canary_milestone("code-917", "label-584");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
