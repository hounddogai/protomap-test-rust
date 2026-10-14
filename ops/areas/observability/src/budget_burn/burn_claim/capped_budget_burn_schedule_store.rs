use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped budget burn schedules by code.
pub trait CappedBudgetBurnScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedBudgetBurnSchedule>;
    fn save(&self, item: &CappedBudgetBurnSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedBudgetBurnSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedBudgetBurnScheduleStore {
    pub items: RefCell<BTreeMap<String, CappedBudgetBurnSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedBudgetBurnScheduleStore for MemoryCappedBudgetBurnScheduleStore {
    fn find(&self, code: &str) -> Option<CappedBudgetBurnSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedBudgetBurnSchedule) {
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

    fn list(&self) -> Vec<CappedBudgetBurnSchedule> {
        let mut out: Vec<CappedBudgetBurnSchedule> = Vec::new();
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

pub fn new_memory_capped_budget_burn_schedule_store() -> Rc<dyn CappedBudgetBurnScheduleStore> {
    Rc::new(MemoryCappedBudgetBurnScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedBudgetBurnScheduleStore {
    pub inner: Rc<dyn CappedBudgetBurnScheduleStore>,
    pub cache: RefCell<cache::TtlCache<CappedBudgetBurnSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedBudgetBurnScheduleStore for CachedCappedBudgetBurnScheduleStore {
    fn find(&self, code: &str) -> Option<CappedBudgetBurnSchedule> {
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

    fn save(&self, item: &CappedBudgetBurnSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedBudgetBurnSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_capped_budget_burn_schedule_store(
    inner: Rc<dyn CappedBudgetBurnScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedBudgetBurnScheduleStore> {
    Rc::new(CachedCappedBudgetBurnScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedBudgetBurnSchedule>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped budget burn schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnScheduleQuery {
    pub segment: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedBudgetBurnScheduleQuery {
    pub fn matches(&self, item: &CappedBudgetBurnSchedule) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedBudgetBurnSchedule]) -> Vec<CappedBudgetBurnSchedule> {
        let matched = collections::filter(items, |candidate: &CappedBudgetBurnSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_budget_burn_schedules(
    store: Rc<dyn CappedBudgetBurnScheduleStore>,
    query: &CappedBudgetBurnScheduleQuery,
) -> Vec<CappedBudgetBurnSchedule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedBudgetBurnScheduleStore> = new_memory_capped_budget_burn_schedule_store();
        let item = new_capped_budget_burn_schedule("code-728", "label-321");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
