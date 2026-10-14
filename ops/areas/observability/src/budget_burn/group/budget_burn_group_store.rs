use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps budget burn groups by code.
pub trait BudgetBurnGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BudgetBurnGroup>;
    fn save(&self, item: &BudgetBurnGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BudgetBurnGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBudgetBurnGroupStore {
    pub items: RefCell<BTreeMap<String, BudgetBurnGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl BudgetBurnGroupStore for MemoryBudgetBurnGroupStore {
    fn find(&self, code: &str) -> Option<BudgetBurnGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BudgetBurnGroup) {
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

    fn list(&self) -> Vec<BudgetBurnGroup> {
        let mut out: Vec<BudgetBurnGroup> = Vec::new();
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

pub fn new_memory_budget_burn_group_store() -> Rc<dyn BudgetBurnGroupStore> {
    Rc::new(MemoryBudgetBurnGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBudgetBurnGroupStore {
    pub inner: Rc<dyn BudgetBurnGroupStore>,
    pub cache: RefCell<cache::TtlCache<BudgetBurnGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BudgetBurnGroupStore for CachedBudgetBurnGroupStore {
    fn find(&self, code: &str) -> Option<BudgetBurnGroup> {
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

    fn save(&self, item: &BudgetBurnGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BudgetBurnGroup> {
        self.inner.list()
    }
}

pub fn new_cached_budget_burn_group_store(
    inner: Rc<dyn BudgetBurnGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BudgetBurnGroupStore> {
    Rc::new(CachedBudgetBurnGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BudgetBurnGroup>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects budget burn groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnGroupQuery {
    pub title: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BudgetBurnGroupQuery {
    pub fn matches(&self, item: &BudgetBurnGroup) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BudgetBurnGroup]) -> Vec<BudgetBurnGroup> {
        let matched = collections::filter(items, |candidate: &BudgetBurnGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_budget_burn_groups(
    store: Rc<dyn BudgetBurnGroupStore>,
    query: &BudgetBurnGroupQuery,
) -> Vec<BudgetBurnGroup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BudgetBurnGroupStore> = new_memory_budget_burn_group_store();
        let item = new_budget_burn_group("code-932", "title-360");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
