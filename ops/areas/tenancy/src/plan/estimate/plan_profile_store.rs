use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps plan profiles by code.
pub trait PlanProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PlanProfile>;
    fn save(&self, item: &PlanProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PlanProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPlanProfileStore {
    pub items: RefCell<BTreeMap<String, PlanProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl PlanProfileStore for MemoryPlanProfileStore {
    fn find(&self, code: &str) -> Option<PlanProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PlanProfile) {
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

    fn list(&self) -> Vec<PlanProfile> {
        let mut out: Vec<PlanProfile> = Vec::new();
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

pub fn new_memory_plan_profile_store() -> Rc<dyn PlanProfileStore> {
    Rc::new(MemoryPlanProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPlanProfileStore {
    pub inner: Rc<dyn PlanProfileStore>,
    pub cache: RefCell<cache::TtlCache<PlanProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PlanProfileStore for CachedPlanProfileStore {
    fn find(&self, code: &str) -> Option<PlanProfile> {
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

    fn save(&self, item: &PlanProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PlanProfile> {
        self.inner.list()
    }
}

pub fn new_cached_plan_profile_store(
    inner: Rc<dyn PlanProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PlanProfileStore> {
    Rc::new(CachedPlanProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PlanProfile>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects plan profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanProfileQuery {
    pub label: String,
    pub min_revision: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PlanProfileQuery {
    pub fn matches(&self, item: &PlanProfile) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.revision < self.min_revision {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PlanProfile]) -> Vec<PlanProfile> {
        let matched = collections::filter(items, |candidate: &PlanProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_plan_profiles(store: Rc<dyn PlanProfileStore>, query: &PlanProfileQuery) -> Vec<PlanProfile> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PlanProfileStore> = new_memory_plan_profile_store();
        let item = new_plan_profile("code-194", "label-337");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
