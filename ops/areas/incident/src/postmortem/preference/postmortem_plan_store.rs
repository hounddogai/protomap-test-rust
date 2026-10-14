use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps postmortem plans by code.
pub trait PostmortemPlanStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PostmortemPlan>;
    fn save(&self, item: &PostmortemPlan);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PostmortemPlan>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPostmortemPlanStore {
    pub items: RefCell<BTreeMap<String, PostmortemPlan>>,
    pub order: RefCell<Vec<String>>,
}

impl PostmortemPlanStore for MemoryPostmortemPlanStore {
    fn find(&self, code: &str) -> Option<PostmortemPlan> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PostmortemPlan) {
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

    fn list(&self) -> Vec<PostmortemPlan> {
        let mut out: Vec<PostmortemPlan> = Vec::new();
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

pub fn new_memory_postmortem_plan_store() -> Rc<dyn PostmortemPlanStore> {
    Rc::new(MemoryPostmortemPlanStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPostmortemPlanStore {
    pub inner: Rc<dyn PostmortemPlanStore>,
    pub cache: RefCell<cache::TtlCache<PostmortemPlan>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PostmortemPlanStore for CachedPostmortemPlanStore {
    fn find(&self, code: &str) -> Option<PostmortemPlan> {
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

    fn save(&self, item: &PostmortemPlan) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PostmortemPlan> {
        self.inner.list()
    }
}

pub fn new_cached_postmortem_plan_store(
    inner: Rc<dyn PostmortemPlanStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PostmortemPlanStore> {
    Rc::new(CachedPostmortemPlanStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PostmortemPlan>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects postmortem plans. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemPlanQuery {
    pub channel: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PostmortemPlanQuery {
    pub fn matches(&self, item: &PostmortemPlan) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PostmortemPlan]) -> Vec<PostmortemPlan> {
        let matched = collections::filter(items, |candidate: &PostmortemPlan| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_postmortem_plans(store: Rc<dyn PostmortemPlanStore>, query: &PostmortemPlanQuery) -> Vec<PostmortemPlan> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PostmortemPlanStore> = new_memory_postmortem_plan_store();
        let item = new_postmortem_plan("code-154", "name-136");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
