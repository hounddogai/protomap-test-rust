use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default timeline budgets by code.
pub trait DefaultTimelineBudgetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultTimelineBudget>;
    fn save(&self, item: &DefaultTimelineBudget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultTimelineBudget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultTimelineBudgetStore {
    pub items: RefCell<BTreeMap<String, DefaultTimelineBudget>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultTimelineBudgetStore for MemoryDefaultTimelineBudgetStore {
    fn find(&self, code: &str) -> Option<DefaultTimelineBudget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultTimelineBudget) {
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

    fn list(&self) -> Vec<DefaultTimelineBudget> {
        let mut out: Vec<DefaultTimelineBudget> = Vec::new();
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

pub fn new_memory_default_timeline_budget_store() -> Rc<dyn DefaultTimelineBudgetStore> {
    Rc::new(MemoryDefaultTimelineBudgetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultTimelineBudgetStore {
    pub inner: Rc<dyn DefaultTimelineBudgetStore>,
    pub cache: RefCell<cache::TtlCache<DefaultTimelineBudget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultTimelineBudgetStore for CachedDefaultTimelineBudgetStore {
    fn find(&self, code: &str) -> Option<DefaultTimelineBudget> {
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

    fn save(&self, item: &DefaultTimelineBudget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultTimelineBudget> {
        self.inner.list()
    }
}

pub fn new_cached_default_timeline_budget_store(
    inner: Rc<dyn DefaultTimelineBudgetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultTimelineBudgetStore> {
    Rc::new(CachedDefaultTimelineBudgetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultTimelineBudget>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default timeline budgets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineBudgetQuery {
    pub label: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultTimelineBudgetQuery {
    pub fn matches(&self, item: &DefaultTimelineBudget) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultTimelineBudget]) -> Vec<DefaultTimelineBudget> {
        let matched = collections::filter(items, |candidate: &DefaultTimelineBudget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_timeline_budgets(
    store: Rc<dyn DefaultTimelineBudgetStore>,
    query: &DefaultTimelineBudgetQuery,
) -> Vec<DefaultTimelineBudget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DefaultTimelineBudgetStore> = new_memory_default_timeline_budget_store();
        let item = new_default_timeline_budget("code-782", "label-781");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
