use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled budget burn notices by code.
pub trait ScheduledBudgetBurnNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledBudgetBurnNotice>;
    fn save(&self, item: &ScheduledBudgetBurnNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledBudgetBurnNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledBudgetBurnNoticeStore {
    pub items: RefCell<BTreeMap<String, ScheduledBudgetBurnNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledBudgetBurnNoticeStore for MemoryScheduledBudgetBurnNoticeStore {
    fn find(&self, code: &str) -> Option<ScheduledBudgetBurnNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledBudgetBurnNotice) {
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

    fn list(&self) -> Vec<ScheduledBudgetBurnNotice> {
        let mut out: Vec<ScheduledBudgetBurnNotice> = Vec::new();
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

pub fn new_memory_scheduled_budget_burn_notice_store() -> Rc<dyn ScheduledBudgetBurnNoticeStore> {
    Rc::new(MemoryScheduledBudgetBurnNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledBudgetBurnNoticeStore {
    pub inner: Rc<dyn ScheduledBudgetBurnNoticeStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledBudgetBurnNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledBudgetBurnNoticeStore for CachedScheduledBudgetBurnNoticeStore {
    fn find(&self, code: &str) -> Option<ScheduledBudgetBurnNotice> {
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

    fn save(&self, item: &ScheduledBudgetBurnNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledBudgetBurnNotice> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_budget_burn_notice_store(
    inner: Rc<dyn ScheduledBudgetBurnNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledBudgetBurnNoticeStore> {
    Rc::new(CachedScheduledBudgetBurnNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledBudgetBurnNotice>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled budget burn notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBudgetBurnNoticeQuery {
    pub region: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledBudgetBurnNoticeQuery {
    pub fn matches(&self, item: &ScheduledBudgetBurnNotice) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledBudgetBurnNotice]) -> Vec<ScheduledBudgetBurnNotice> {
        let matched = collections::filter(items, |candidate: &ScheduledBudgetBurnNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_budget_burn_notices(
    store: Rc<dyn ScheduledBudgetBurnNoticeStore>,
    query: &ScheduledBudgetBurnNoticeQuery,
) -> Vec<ScheduledBudgetBurnNotice> {
    query.apply(&store.list())
}
