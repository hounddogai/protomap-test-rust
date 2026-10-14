use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps egress groups by code.
pub trait EgressGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EgressGroup>;
    fn save(&self, item: &EgressGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EgressGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEgressGroupStore {
    pub items: RefCell<BTreeMap<String, EgressGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl EgressGroupStore for MemoryEgressGroupStore {
    fn find(&self, code: &str) -> Option<EgressGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EgressGroup) {
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

    fn list(&self) -> Vec<EgressGroup> {
        let mut out: Vec<EgressGroup> = Vec::new();
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

pub fn new_memory_egress_group_store() -> Rc<dyn EgressGroupStore> {
    Rc::new(MemoryEgressGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEgressGroupStore {
    pub inner: Rc<dyn EgressGroupStore>,
    pub cache: RefCell<cache::TtlCache<EgressGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EgressGroupStore for CachedEgressGroupStore {
    fn find(&self, code: &str) -> Option<EgressGroup> {
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

    fn save(&self, item: &EgressGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EgressGroup> {
        self.inner.list()
    }
}

pub fn new_cached_egress_group_store(
    inner: Rc<dyn EgressGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EgressGroupStore> {
    Rc::new(CachedEgressGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EgressGroup>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects egress groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressGroupQuery {
    pub label: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EgressGroupQuery {
    pub fn matches(&self, item: &EgressGroup) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[EgressGroup]) -> Vec<EgressGroup> {
        let matched = collections::filter(items, |candidate: &EgressGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_egress_groups(store: Rc<dyn EgressGroupStore>, query: &EgressGroupQuery) -> Vec<EgressGroup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn EgressGroupStore> = new_memory_egress_group_store();
        let item = new_egress_group("code-580", "label-246");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
