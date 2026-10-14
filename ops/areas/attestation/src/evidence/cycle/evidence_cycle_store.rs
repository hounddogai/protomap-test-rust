use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps evidence cycles by code.
pub trait EvidenceCycleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EvidenceCycle>;
    fn save(&self, item: &EvidenceCycle);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EvidenceCycle>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEvidenceCycleStore {
    pub items: RefCell<BTreeMap<String, EvidenceCycle>>,
    pub order: RefCell<Vec<String>>,
}

impl EvidenceCycleStore for MemoryEvidenceCycleStore {
    fn find(&self, code: &str) -> Option<EvidenceCycle> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EvidenceCycle) {
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

    fn list(&self) -> Vec<EvidenceCycle> {
        let mut out: Vec<EvidenceCycle> = Vec::new();
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

pub fn new_memory_evidence_cycle_store() -> Rc<dyn EvidenceCycleStore> {
    Rc::new(MemoryEvidenceCycleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEvidenceCycleStore {
    pub inner: Rc<dyn EvidenceCycleStore>,
    pub cache: RefCell<cache::TtlCache<EvidenceCycle>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EvidenceCycleStore for CachedEvidenceCycleStore {
    fn find(&self, code: &str) -> Option<EvidenceCycle> {
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

    fn save(&self, item: &EvidenceCycle) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EvidenceCycle> {
        self.inner.list()
    }
}

pub fn new_cached_evidence_cycle_store(
    inner: Rc<dyn EvidenceCycleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EvidenceCycleStore> {
    Rc::new(CachedEvidenceCycleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EvidenceCycle>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects evidence cycles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceCycleQuery {
    pub segment: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EvidenceCycleQuery {
    pub fn matches(&self, item: &EvidenceCycle) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
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

    pub fn apply(&self, items: &[EvidenceCycle]) -> Vec<EvidenceCycle> {
        let matched = collections::filter(items, |candidate: &EvidenceCycle| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_evidence_cycles(store: Rc<dyn EvidenceCycleStore>, query: &EvidenceCycleQuery) -> Vec<EvidenceCycle> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn EvidenceCycleStore> = new_memory_evidence_cycle_store();
        let item = new_evidence_cycle("code-611", "name-85");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
