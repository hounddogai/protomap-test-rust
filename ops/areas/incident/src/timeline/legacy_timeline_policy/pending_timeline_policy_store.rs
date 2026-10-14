use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending timeline policies by code.
pub trait PendingTimelinePolicyStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingTimelinePolicy>;
    fn save(&self, item: &PendingTimelinePolicy);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingTimelinePolicy>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingTimelinePolicyStore {
    pub items: RefCell<BTreeMap<String, PendingTimelinePolicy>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingTimelinePolicyStore for MemoryPendingTimelinePolicyStore {
    fn find(&self, code: &str) -> Option<PendingTimelinePolicy> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingTimelinePolicy) {
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

    fn list(&self) -> Vec<PendingTimelinePolicy> {
        let mut out: Vec<PendingTimelinePolicy> = Vec::new();
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

pub fn new_memory_pending_timeline_policy_store() -> Rc<dyn PendingTimelinePolicyStore> {
    Rc::new(MemoryPendingTimelinePolicyStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingTimelinePolicyStore {
    pub inner: Rc<dyn PendingTimelinePolicyStore>,
    pub cache: RefCell<cache::TtlCache<PendingTimelinePolicy>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingTimelinePolicyStore for CachedPendingTimelinePolicyStore {
    fn find(&self, code: &str) -> Option<PendingTimelinePolicy> {
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

    fn save(&self, item: &PendingTimelinePolicy) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingTimelinePolicy> {
        self.inner.list()
    }
}

pub fn new_cached_pending_timeline_policy_store(
    inner: Rc<dyn PendingTimelinePolicyStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingTimelinePolicyStore> {
    Rc::new(CachedPendingTimelinePolicyStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingTimelinePolicy>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending timeline policies. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelinePolicyQuery {
    pub label: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingTimelinePolicyQuery {
    pub fn matches(&self, item: &PendingTimelinePolicy) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.margin < self.min_margin {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PendingTimelinePolicy]) -> Vec<PendingTimelinePolicy> {
        let matched = collections::filter(items, |candidate: &PendingTimelinePolicy| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_timeline_policies(
    store: Rc<dyn PendingTimelinePolicyStore>,
    query: &PendingTimelinePolicyQuery,
) -> Vec<PendingTimelinePolicy> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PendingTimelinePolicyStore> = new_memory_pending_timeline_policy_store();
        let item = new_pending_timeline_policy("code-261", "label-483");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
