use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal plan profiles by code.
pub trait SeasonalPlanProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalPlanProfile>;
    fn save(&self, item: &SeasonalPlanProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalPlanProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalPlanProfileStore {
    pub items: RefCell<BTreeMap<String, SeasonalPlanProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalPlanProfileStore for MemorySeasonalPlanProfileStore {
    fn find(&self, code: &str) -> Option<SeasonalPlanProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalPlanProfile) {
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

    fn list(&self) -> Vec<SeasonalPlanProfile> {
        let mut out: Vec<SeasonalPlanProfile> = Vec::new();
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

pub fn new_memory_seasonal_plan_profile_store() -> Rc<dyn SeasonalPlanProfileStore> {
    Rc::new(MemorySeasonalPlanProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalPlanProfileStore {
    pub inner: Rc<dyn SeasonalPlanProfileStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalPlanProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalPlanProfileStore for CachedSeasonalPlanProfileStore {
    fn find(&self, code: &str) -> Option<SeasonalPlanProfile> {
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

    fn save(&self, item: &SeasonalPlanProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalPlanProfile> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_plan_profile_store(
    inner: Rc<dyn SeasonalPlanProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalPlanProfileStore> {
    Rc::new(CachedSeasonalPlanProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalPlanProfile>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal plan profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPlanProfileQuery {
    pub category: String,
    pub min_retries: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalPlanProfileQuery {
    pub fn matches(&self, item: &SeasonalPlanProfile) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.retries < self.min_retries {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalPlanProfile]) -> Vec<SeasonalPlanProfile> {
        let matched = collections::filter(items, |candidate: &SeasonalPlanProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_plan_profiles(
    store: Rc<dyn SeasonalPlanProfileStore>,
    query: &SeasonalPlanProfileQuery,
) -> Vec<SeasonalPlanProfile> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SeasonalPlanProfileStore> = new_memory_seasonal_plan_profile_store();
        let item = new_seasonal_plan_profile("code-826", "name-563");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
