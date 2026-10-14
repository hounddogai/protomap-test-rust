use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps policy summaries by code.
pub trait PolicySummaryStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PolicySummary>;
    fn save(&self, item: &PolicySummary);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PolicySummary>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPolicySummaryStore {
    pub items: RefCell<BTreeMap<String, PolicySummary>>,
    pub order: RefCell<Vec<String>>,
}

impl PolicySummaryStore for MemoryPolicySummaryStore {
    fn find(&self, code: &str) -> Option<PolicySummary> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PolicySummary) {
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

    fn list(&self) -> Vec<PolicySummary> {
        let mut out: Vec<PolicySummary> = Vec::new();
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

pub fn new_memory_policy_summary_store() -> Rc<dyn PolicySummaryStore> {
    Rc::new(MemoryPolicySummaryStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPolicySummaryStore {
    pub inner: Rc<dyn PolicySummaryStore>,
    pub cache: RefCell<cache::TtlCache<PolicySummary>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PolicySummaryStore for CachedPolicySummaryStore {
    fn find(&self, code: &str) -> Option<PolicySummary> {
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

    fn save(&self, item: &PolicySummary) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PolicySummary> {
        self.inner.list()
    }
}

pub fn new_cached_policy_summary_store(
    inner: Rc<dyn PolicySummaryStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PolicySummaryStore> {
    Rc::new(CachedPolicySummaryStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PolicySummary>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects policy summaries. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicySummaryQuery {
    pub channel: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PolicySummaryQuery {
    pub fn matches(&self, item: &PolicySummary) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
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

    pub fn apply(&self, items: &[PolicySummary]) -> Vec<PolicySummary> {
        let matched = collections::filter(items, |candidate: &PolicySummary| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_policy_summaries(store: Rc<dyn PolicySummaryStore>, query: &PolicySummaryQuery) -> Vec<PolicySummary> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PolicySummaryStore> = new_memory_policy_summary_store();
        let item = new_policy_summary("code-252", "title-590");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
