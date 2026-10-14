use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict forecast milestones by code.
pub trait StrictForecastMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictForecastMilestone>;
    fn save(&self, item: &StrictForecastMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictForecastMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictForecastMilestoneStore {
    pub items: RefCell<BTreeMap<String, StrictForecastMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictForecastMilestoneStore for MemoryStrictForecastMilestoneStore {
    fn find(&self, code: &str) -> Option<StrictForecastMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictForecastMilestone) {
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

    fn list(&self) -> Vec<StrictForecastMilestone> {
        let mut out: Vec<StrictForecastMilestone> = Vec::new();
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

pub fn new_memory_strict_forecast_milestone_store() -> Rc<dyn StrictForecastMilestoneStore> {
    Rc::new(MemoryStrictForecastMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictForecastMilestoneStore {
    pub inner: Rc<dyn StrictForecastMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<StrictForecastMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictForecastMilestoneStore for CachedStrictForecastMilestoneStore {
    fn find(&self, code: &str) -> Option<StrictForecastMilestone> {
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

    fn save(&self, item: &StrictForecastMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictForecastMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_strict_forecast_milestone_store(
    inner: Rc<dyn StrictForecastMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictForecastMilestoneStore> {
    Rc::new(CachedStrictForecastMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictForecastMilestone>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict forecast milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastMilestoneQuery {
    pub region: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictForecastMilestoneQuery {
    pub fn matches(&self, item: &StrictForecastMilestone) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictForecastMilestone]) -> Vec<StrictForecastMilestone> {
        let matched = collections::filter(items, |candidate: &StrictForecastMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_forecast_milestones(
    store: Rc<dyn StrictForecastMilestoneStore>,
    query: &StrictForecastMilestoneQuery,
) -> Vec<StrictForecastMilestone> {
    query.apply(&store.list())
}
