use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional peering budgets by code.
pub trait RegionalPeeringBudgetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalPeeringBudget>;
    fn save(&self, item: &RegionalPeeringBudget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalPeeringBudget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalPeeringBudgetStore {
    pub items: RefCell<BTreeMap<String, RegionalPeeringBudget>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalPeeringBudgetStore for MemoryRegionalPeeringBudgetStore {
    fn find(&self, code: &str) -> Option<RegionalPeeringBudget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalPeeringBudget) {
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

    fn list(&self) -> Vec<RegionalPeeringBudget> {
        let mut out: Vec<RegionalPeeringBudget> = Vec::new();
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

pub fn new_memory_regional_peering_budget_store() -> Rc<dyn RegionalPeeringBudgetStore> {
    Rc::new(MemoryRegionalPeeringBudgetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalPeeringBudgetStore {
    pub inner: Rc<dyn RegionalPeeringBudgetStore>,
    pub cache: RefCell<cache::TtlCache<RegionalPeeringBudget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalPeeringBudgetStore for CachedRegionalPeeringBudgetStore {
    fn find(&self, code: &str) -> Option<RegionalPeeringBudget> {
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

    fn save(&self, item: &RegionalPeeringBudget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalPeeringBudget> {
        self.inner.list()
    }
}

pub fn new_cached_regional_peering_budget_store(
    inner: Rc<dyn RegionalPeeringBudgetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalPeeringBudgetStore> {
    Rc::new(CachedRegionalPeeringBudgetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalPeeringBudget>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional peering budgets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringBudgetQuery {
    pub segment: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalPeeringBudgetQuery {
    pub fn matches(&self, item: &RegionalPeeringBudget) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalPeeringBudget]) -> Vec<RegionalPeeringBudget> {
        let matched = collections::filter(items, |candidate: &RegionalPeeringBudget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_peering_budgets(
    store: Rc<dyn RegionalPeeringBudgetStore>,
    query: &RegionalPeeringBudgetQuery,
) -> Vec<RegionalPeeringBudget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalPeeringBudgetStore> = new_memory_regional_peering_budget_store();
        let item = new_regional_peering_budget("code-492", "title-294");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
