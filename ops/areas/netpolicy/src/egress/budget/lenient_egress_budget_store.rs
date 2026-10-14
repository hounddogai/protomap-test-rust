use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient egress budgets by code.
pub trait LenientEgressBudgetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientEgressBudget>;
    fn save(&self, item: &LenientEgressBudget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientEgressBudget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientEgressBudgetStore {
    pub items: RefCell<BTreeMap<String, LenientEgressBudget>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientEgressBudgetStore for MemoryLenientEgressBudgetStore {
    fn find(&self, code: &str) -> Option<LenientEgressBudget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientEgressBudget) {
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

    fn list(&self) -> Vec<LenientEgressBudget> {
        let mut out: Vec<LenientEgressBudget> = Vec::new();
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

pub fn new_memory_lenient_egress_budget_store() -> Rc<dyn LenientEgressBudgetStore> {
    Rc::new(MemoryLenientEgressBudgetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientEgressBudgetStore {
    pub inner: Rc<dyn LenientEgressBudgetStore>,
    pub cache: RefCell<cache::TtlCache<LenientEgressBudget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientEgressBudgetStore for CachedLenientEgressBudgetStore {
    fn find(&self, code: &str) -> Option<LenientEgressBudget> {
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

    fn save(&self, item: &LenientEgressBudget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientEgressBudget> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_egress_budget_store(
    inner: Rc<dyn LenientEgressBudgetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientEgressBudgetStore> {
    Rc::new(CachedLenientEgressBudgetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientEgressBudget>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient egress budgets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressBudgetQuery {
    pub title: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientEgressBudgetQuery {
    pub fn matches(&self, item: &LenientEgressBudget) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[LenientEgressBudget]) -> Vec<LenientEgressBudget> {
        let matched = collections::filter(items, |candidate: &LenientEgressBudget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_egress_budgets(
    store: Rc<dyn LenientEgressBudgetStore>,
    query: &LenientEgressBudgetQuery,
) -> Vec<LenientEgressBudget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientEgressBudgetStore> = new_memory_lenient_egress_budget_store();
        let item = new_lenient_egress_budget("code-951", "title-909");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
