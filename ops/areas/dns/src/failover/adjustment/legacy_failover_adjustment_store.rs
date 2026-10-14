use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy failover adjustments by code.
pub trait LegacyFailoverAdjustmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyFailoverAdjustment>;
    fn save(&self, item: &LegacyFailoverAdjustment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyFailoverAdjustment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyFailoverAdjustmentStore {
    pub items: RefCell<BTreeMap<String, LegacyFailoverAdjustment>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyFailoverAdjustmentStore for MemoryLegacyFailoverAdjustmentStore {
    fn find(&self, code: &str) -> Option<LegacyFailoverAdjustment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyFailoverAdjustment) {
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

    fn list(&self) -> Vec<LegacyFailoverAdjustment> {
        let mut out: Vec<LegacyFailoverAdjustment> = Vec::new();
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

pub fn new_memory_legacy_failover_adjustment_store() -> Rc<dyn LegacyFailoverAdjustmentStore> {
    Rc::new(MemoryLegacyFailoverAdjustmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyFailoverAdjustmentStore {
    pub inner: Rc<dyn LegacyFailoverAdjustmentStore>,
    pub cache: RefCell<cache::TtlCache<LegacyFailoverAdjustment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyFailoverAdjustmentStore for CachedLegacyFailoverAdjustmentStore {
    fn find(&self, code: &str) -> Option<LegacyFailoverAdjustment> {
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

    fn save(&self, item: &LegacyFailoverAdjustment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyFailoverAdjustment> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_failover_adjustment_store(
    inner: Rc<dyn LegacyFailoverAdjustmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyFailoverAdjustmentStore> {
    Rc::new(CachedLegacyFailoverAdjustmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyFailoverAdjustment>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy failover adjustments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverAdjustmentQuery {
    pub region: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyFailoverAdjustmentQuery {
    pub fn matches(&self, item: &LegacyFailoverAdjustment) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyFailoverAdjustment]) -> Vec<LegacyFailoverAdjustment> {
        let matched = collections::filter(items, |candidate: &LegacyFailoverAdjustment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_failover_adjustments(
    store: Rc<dyn LegacyFailoverAdjustmentStore>,
    query: &LegacyFailoverAdjustmentQuery,
) -> Vec<LegacyFailoverAdjustment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyFailoverAdjustmentStore> = new_memory_legacy_failover_adjustment_store();
        let item = new_legacy_failover_adjustment("code-418", "name-399");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
