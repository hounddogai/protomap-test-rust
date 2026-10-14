use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps budget burn schedules by code.
pub trait BudgetBurnScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BudgetBurnSchedule>;
    fn save(&self, item: &BudgetBurnSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BudgetBurnSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBudgetBurnScheduleStore {
    pub items: RefCell<BTreeMap<String, BudgetBurnSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl BudgetBurnScheduleStore for MemoryBudgetBurnScheduleStore {
    fn find(&self, code: &str) -> Option<BudgetBurnSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BudgetBurnSchedule) {
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

    fn list(&self) -> Vec<BudgetBurnSchedule> {
        let mut out: Vec<BudgetBurnSchedule> = Vec::new();
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

pub fn new_memory_budget_burn_schedule_store() -> Rc<dyn BudgetBurnScheduleStore> {
    Rc::new(MemoryBudgetBurnScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBudgetBurnScheduleStore {
    pub inner: Rc<dyn BudgetBurnScheduleStore>,
    pub cache: RefCell<cache::TtlCache<BudgetBurnSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BudgetBurnScheduleStore for CachedBudgetBurnScheduleStore {
    fn find(&self, code: &str) -> Option<BudgetBurnSchedule> {
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

    fn save(&self, item: &BudgetBurnSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BudgetBurnSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_budget_burn_schedule_store(
    inner: Rc<dyn BudgetBurnScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BudgetBurnScheduleStore> {
    Rc::new(CachedBudgetBurnScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BudgetBurnSchedule>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects budget burn schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnScheduleQuery {
    pub scope: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BudgetBurnScheduleQuery {
    pub fn matches(&self, item: &BudgetBurnSchedule) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BudgetBurnSchedule]) -> Vec<BudgetBurnSchedule> {
        let matched = collections::filter(items, |candidate: &BudgetBurnSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_budget_burn_schedules(
    store: Rc<dyn BudgetBurnScheduleStore>,
    query: &BudgetBurnScheduleQuery,
) -> Vec<BudgetBurnSchedule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BudgetBurnScheduleStore> = new_memory_budget_burn_schedule_store();
        let item = new_budget_burn_schedule("code-921", "name-39");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
