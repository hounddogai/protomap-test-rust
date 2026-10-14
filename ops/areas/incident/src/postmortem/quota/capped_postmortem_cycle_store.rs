use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped postmortem cycles by code.
pub trait CappedPostmortemCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedPostmortemCycle>;
    fn save(&self, item: &CappedPostmortemCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedPostmortemCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedPostmortemCycleStore {
    pub items: RefCell<BTreeMap<String, CappedPostmortemCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedPostmortemCycleStore for MemoryCappedPostmortemCycleStore {
    fn find(&self, code: &str) -> Option<CappedPostmortemCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedPostmortemCycle) {
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

    fn list(&self) -> Vec<CappedPostmortemCycle> {
        let mut out: Vec<CappedPostmortemCycle> = Vec::new();
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

pub fn new_memory_capped_postmortem_cycle_store() -> Rc<dyn CappedPostmortemCycleStore> {
    Rc::new(MemoryCappedPostmortemCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedPostmortemCycleStore {
    pub inner: Rc<dyn CappedPostmortemCycleStore>,
    pub cache: RefCell<cache::TtlCache<CappedPostmortemCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedPostmortemCycleStore for CachedCappedPostmortemCycleStore {
    fn find(&self, code: &str) -> Option<CappedPostmortemCycle> {
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

    fn save(&self, item: &CappedPostmortemCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedPostmortemCycle> {
        self.inner.list()
    }
}

pub fn new_cached_capped_postmortem_cycle_store(
    inner: Rc<dyn CappedPostmortemCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedPostmortemCycleStore> {
    Rc::new(CachedCappedPostmortemCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedPostmortemCycle>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped postmortem cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPostmortemCycleQuery {
    pub source: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedPostmortemCycleQuery {
    pub fn matches(&self, item: &CappedPostmortemCycle) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CappedPostmortemCycle]) -> Vec<CappedPostmortemCycle> {
        let matched = collections::filter(items, |candidate: &CappedPostmortemCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_postmortem_cycles(
    store: Rc<dyn CappedPostmortemCycleStore>,
    query: &CappedPostmortemCycleQuery,
) -> Vec<CappedPostmortemCycle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedPostmortemCycleStore> = new_memory_capped_postmortem_cycle_store();
        let item = new_capped_postmortem_cycle("code-947", "label-307");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
