use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled evidence tiers by code.
pub trait ScheduledEvidenceTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledEvidenceTier>;
    fn save(&self, item: &ScheduledEvidenceTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledEvidenceTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledEvidenceTierStore {
    pub items: RefCell<BTreeMap<String, ScheduledEvidenceTier>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledEvidenceTierStore for MemoryScheduledEvidenceTierStore {
    fn find(&self, code: &str) -> Option<ScheduledEvidenceTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledEvidenceTier) {
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

    fn list(&self) -> Vec<ScheduledEvidenceTier> {
        let mut out: Vec<ScheduledEvidenceTier> = Vec::new();
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

pub fn new_memory_scheduled_evidence_tier_store() -> Rc<dyn ScheduledEvidenceTierStore> {
    Rc::new(MemoryScheduledEvidenceTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledEvidenceTierStore {
    pub inner: Rc<dyn ScheduledEvidenceTierStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledEvidenceTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledEvidenceTierStore for CachedScheduledEvidenceTierStore {
    fn find(&self, code: &str) -> Option<ScheduledEvidenceTier> {
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

    fn save(&self, item: &ScheduledEvidenceTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledEvidenceTier> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_evidence_tier_store(
    inner: Rc<dyn ScheduledEvidenceTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledEvidenceTierStore> {
    Rc::new(CachedScheduledEvidenceTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledEvidenceTier>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled evidence tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEvidenceTierQuery {
    pub name: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledEvidenceTierQuery {
    pub fn matches(&self, item: &ScheduledEvidenceTier) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledEvidenceTier]) -> Vec<ScheduledEvidenceTier> {
        let matched = collections::filter(items, |candidate: &ScheduledEvidenceTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_evidence_tiers(
    store: Rc<dyn ScheduledEvidenceTierStore>,
    query: &ScheduledEvidenceTierQuery,
) -> Vec<ScheduledEvidenceTier> {
    query.apply(&store.list())
}
