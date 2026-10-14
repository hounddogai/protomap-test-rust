use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps peering groups by code.
pub trait PeeringGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PeeringGroup>;
    fn save(&self, item: &PeeringGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PeeringGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPeeringGroupStore {
    pub items: RefCell<BTreeMap<String, PeeringGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl PeeringGroupStore for MemoryPeeringGroupStore {
    fn find(&self, code: &str) -> Option<PeeringGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PeeringGroup) {
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

    fn list(&self) -> Vec<PeeringGroup> {
        let mut out: Vec<PeeringGroup> = Vec::new();
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

pub fn new_memory_peering_group_store() -> Rc<dyn PeeringGroupStore> {
    Rc::new(MemoryPeeringGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPeeringGroupStore {
    pub inner: Rc<dyn PeeringGroupStore>,
    pub cache: RefCell<cache::TtlCache<PeeringGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PeeringGroupStore for CachedPeeringGroupStore {
    fn find(&self, code: &str) -> Option<PeeringGroup> {
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

    fn save(&self, item: &PeeringGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PeeringGroup> {
        self.inner.list()
    }
}

pub fn new_cached_peering_group_store(
    inner: Rc<dyn PeeringGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PeeringGroupStore> {
    Rc::new(CachedPeeringGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PeeringGroup>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects peering groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringGroupQuery {
    pub title: String,
    pub min_updated_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PeeringGroupQuery {
    pub fn matches(&self, item: &PeeringGroup) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.updated_at < self.min_updated_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PeeringGroup]) -> Vec<PeeringGroup> {
        let matched = collections::filter(items, |candidate: &PeeringGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_peering_groups(store: Rc<dyn PeeringGroupStore>, query: &PeeringGroupQuery) -> Vec<PeeringGroup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PeeringGroupStore> = new_memory_peering_group_store();
        let item = new_peering_group("code-282", "title-481");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
