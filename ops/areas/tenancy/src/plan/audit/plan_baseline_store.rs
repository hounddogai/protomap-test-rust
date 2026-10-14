use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps plan baselines by code.
pub trait PlanBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PlanBaseline>;
    fn save(&self, item: &PlanBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PlanBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPlanBaselineStore {
    pub items: RefCell<BTreeMap<String, PlanBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl PlanBaselineStore for MemoryPlanBaselineStore {
    fn find(&self, code: &str) -> Option<PlanBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PlanBaseline) {
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

    fn list(&self) -> Vec<PlanBaseline> {
        let mut out: Vec<PlanBaseline> = Vec::new();
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

pub fn new_memory_plan_baseline_store() -> Rc<dyn PlanBaselineStore> {
    Rc::new(MemoryPlanBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPlanBaselineStore {
    pub inner: Rc<dyn PlanBaselineStore>,
    pub cache: RefCell<cache::TtlCache<PlanBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PlanBaselineStore for CachedPlanBaselineStore {
    fn find(&self, code: &str) -> Option<PlanBaseline> {
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

    fn save(&self, item: &PlanBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PlanBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_plan_baseline_store(
    inner: Rc<dyn PlanBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PlanBaselineStore> {
    Rc::new(CachedPlanBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PlanBaseline>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects plan baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanBaselineQuery {
    pub scope: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PlanBaselineQuery {
    pub fn matches(&self, item: &PlanBaseline) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
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

    pub fn apply(&self, items: &[PlanBaseline]) -> Vec<PlanBaseline> {
        let matched = collections::filter(items, |candidate: &PlanBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_plan_baselines(store: Rc<dyn PlanBaselineStore>, query: &PlanBaselineQuery) -> Vec<PlanBaseline> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PlanBaselineStore> = new_memory_plan_baseline_store();
        let item = new_plan_baseline("code-641", "label-267");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
