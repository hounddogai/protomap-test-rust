use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered egress milestones by code.
pub trait TieredEgressMilestoneStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredEgressMilestone>;
    fn save(&self, item: &TieredEgressMilestone);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredEgressMilestone>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredEgressMilestoneStore {
    pub items: RefCell<BTreeMap<String, TieredEgressMilestone>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredEgressMilestoneStore for MemoryTieredEgressMilestoneStore {
    fn find(&self, code: &str) -> Option<TieredEgressMilestone> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredEgressMilestone) {
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

    fn list(&self) -> Vec<TieredEgressMilestone> {
        let mut out: Vec<TieredEgressMilestone> = Vec::new();
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

pub fn new_memory_tiered_egress_milestone_store() -> Rc<dyn TieredEgressMilestoneStore> {
    Rc::new(MemoryTieredEgressMilestoneStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredEgressMilestoneStore {
    pub inner: Rc<dyn TieredEgressMilestoneStore>,
    pub cache: RefCell<cache::TtlCache<TieredEgressMilestone>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredEgressMilestoneStore for CachedTieredEgressMilestoneStore {
    fn find(&self, code: &str) -> Option<TieredEgressMilestone> {
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

    fn save(&self, item: &TieredEgressMilestone) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredEgressMilestone> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_egress_milestone_store(
    inner: Rc<dyn TieredEgressMilestoneStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredEgressMilestoneStore> {
    Rc::new(CachedTieredEgressMilestoneStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredEgressMilestone>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered egress milestones. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressMilestoneQuery {
    pub name: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredEgressMilestoneQuery {
    pub fn matches(&self, item: &TieredEgressMilestone) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
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

    pub fn apply(&self, items: &[TieredEgressMilestone]) -> Vec<TieredEgressMilestone> {
        let matched = collections::filter(items, |candidate: &TieredEgressMilestone| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_egress_milestones(
    store: Rc<dyn TieredEgressMilestoneStore>,
    query: &TieredEgressMilestoneQuery,
) -> Vec<TieredEgressMilestone> {
    query.apply(&store.list())
}
