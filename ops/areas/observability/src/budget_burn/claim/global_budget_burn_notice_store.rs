use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global budget burn notices by code.
pub trait GlobalBudgetBurnNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalBudgetBurnNotice>;
    fn save(&self, item: &GlobalBudgetBurnNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalBudgetBurnNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalBudgetBurnNoticeStore {
    pub items: RefCell<BTreeMap<String, GlobalBudgetBurnNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalBudgetBurnNoticeStore for MemoryGlobalBudgetBurnNoticeStore {
    fn find(&self, code: &str) -> Option<GlobalBudgetBurnNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalBudgetBurnNotice) {
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

    fn list(&self) -> Vec<GlobalBudgetBurnNotice> {
        let mut out: Vec<GlobalBudgetBurnNotice> = Vec::new();
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

pub fn new_memory_global_budget_burn_notice_store() -> Rc<dyn GlobalBudgetBurnNoticeStore> {
    Rc::new(MemoryGlobalBudgetBurnNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalBudgetBurnNoticeStore {
    pub inner: Rc<dyn GlobalBudgetBurnNoticeStore>,
    pub cache: RefCell<cache::TtlCache<GlobalBudgetBurnNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalBudgetBurnNoticeStore for CachedGlobalBudgetBurnNoticeStore {
    fn find(&self, code: &str) -> Option<GlobalBudgetBurnNotice> {
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

    fn save(&self, item: &GlobalBudgetBurnNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalBudgetBurnNotice> {
        self.inner.list()
    }
}

pub fn new_cached_global_budget_burn_notice_store(
    inner: Rc<dyn GlobalBudgetBurnNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalBudgetBurnNoticeStore> {
    Rc::new(CachedGlobalBudgetBurnNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalBudgetBurnNotice>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global budget burn notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnNoticeQuery {
    pub cluster: String,
    pub min_margin: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalBudgetBurnNoticeQuery {
    pub fn matches(&self, item: &GlobalBudgetBurnNotice) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[GlobalBudgetBurnNotice]) -> Vec<GlobalBudgetBurnNotice> {
        let matched = collections::filter(items, |candidate: &GlobalBudgetBurnNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_budget_burn_notices(
    store: Rc<dyn GlobalBudgetBurnNoticeStore>,
    query: &GlobalBudgetBurnNoticeQuery,
) -> Vec<GlobalBudgetBurnNotice> {
    query.apply(&store.list())
}
