use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps egress targets by code.
pub trait EgressTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EgressTarget>;
    fn save(&self, item: &EgressTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EgressTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEgressTargetStore {
    pub items: RefCell<BTreeMap<String, EgressTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl EgressTargetStore for MemoryEgressTargetStore {
    fn find(&self, code: &str) -> Option<EgressTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EgressTarget) {
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

    fn list(&self) -> Vec<EgressTarget> {
        let mut out: Vec<EgressTarget> = Vec::new();
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

pub fn new_memory_egress_target_store() -> Rc<dyn EgressTargetStore> {
    Rc::new(MemoryEgressTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEgressTargetStore {
    pub inner: Rc<dyn EgressTargetStore>,
    pub cache: RefCell<cache::TtlCache<EgressTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EgressTargetStore for CachedEgressTargetStore {
    fn find(&self, code: &str) -> Option<EgressTarget> {
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

    fn save(&self, item: &EgressTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EgressTarget> {
        self.inner.list()
    }
}

pub fn new_cached_egress_target_store(
    inner: Rc<dyn EgressTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EgressTargetStore> {
    Rc::new(CachedEgressTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EgressTarget>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects egress targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressTargetQuery {
    pub unit: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EgressTargetQuery {
    pub fn matches(&self, item: &EgressTarget) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
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

    pub fn apply(&self, items: &[EgressTarget]) -> Vec<EgressTarget> {
        let matched = collections::filter(items, |candidate: &EgressTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_egress_targets(store: Rc<dyn EgressTargetStore>, query: &EgressTargetQuery) -> Vec<EgressTarget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn EgressTargetStore> = new_memory_egress_target_store();
        let item = new_egress_target("code-172", "name-382");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
