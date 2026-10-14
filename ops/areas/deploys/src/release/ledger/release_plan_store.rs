use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps release plans by code.
pub trait ReleasePlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ReleasePlan>;
    fn save(&self, item: &ReleasePlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ReleasePlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryReleasePlanStore {
    pub items: RefCell<BTreeMap<String, ReleasePlan>>,
    pub order: RefCell<Vec<String>>,
}

impl ReleasePlanStore for MemoryReleasePlanStore {
    fn find(&self, code: &str) -> Option<ReleasePlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ReleasePlan) {
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

    fn list(&self) -> Vec<ReleasePlan> {
        let mut out: Vec<ReleasePlan> = Vec::new();
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

pub fn new_memory_release_plan_store() -> Rc<dyn ReleasePlanStore> {
    Rc::new(MemoryReleasePlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedReleasePlanStore {
    pub inner: Rc<dyn ReleasePlanStore>,
    pub cache: RefCell<cache::TtlCache<ReleasePlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ReleasePlanStore for CachedReleasePlanStore {
    fn find(&self, code: &str) -> Option<ReleasePlan> {
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

    fn save(&self, item: &ReleasePlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ReleasePlan> {
        self.inner.list()
    }
}

pub fn new_cached_release_plan_store(
    inner: Rc<dyn ReleasePlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ReleasePlanStore> {
    Rc::new(CachedReleasePlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ReleasePlan>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects release plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePlanQuery {
    pub title: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ReleasePlanQuery {
    pub fn matches(&self, item: &ReleasePlan) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ReleasePlan]) -> Vec<ReleasePlan> {
        let matched = collections::filter(items, |candidate: &ReleasePlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_release_plans(store: Rc<dyn ReleasePlanStore>, query: &ReleasePlanQuery) -> Vec<ReleasePlan> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ReleasePlanStore> = new_memory_release_plan_store();
        let item = new_release_plan("code-14", "title-316");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
