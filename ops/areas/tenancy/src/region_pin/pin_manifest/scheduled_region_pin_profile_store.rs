use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled region pin profiles by code.
pub trait ScheduledRegionPinProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledRegionPinProfile>;
    fn save(&self, item: &ScheduledRegionPinProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledRegionPinProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledRegionPinProfileStore {
    pub items: RefCell<BTreeMap<String, ScheduledRegionPinProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledRegionPinProfileStore for MemoryScheduledRegionPinProfileStore {
    fn find(&self, code: &str) -> Option<ScheduledRegionPinProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledRegionPinProfile) {
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

    fn list(&self) -> Vec<ScheduledRegionPinProfile> {
        let mut out: Vec<ScheduledRegionPinProfile> = Vec::new();
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

pub fn new_memory_scheduled_region_pin_profile_store() -> Rc<dyn ScheduledRegionPinProfileStore> {
    Rc::new(MemoryScheduledRegionPinProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledRegionPinProfileStore {
    pub inner: Rc<dyn ScheduledRegionPinProfileStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledRegionPinProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledRegionPinProfileStore for CachedScheduledRegionPinProfileStore {
    fn find(&self, code: &str) -> Option<ScheduledRegionPinProfile> {
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

    fn save(&self, item: &ScheduledRegionPinProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledRegionPinProfile> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_region_pin_profile_store(
    inner: Rc<dyn ScheduledRegionPinProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledRegionPinProfileStore> {
    Rc::new(CachedScheduledRegionPinProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledRegionPinProfile>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled region pin profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinProfileQuery {
    pub name: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledRegionPinProfileQuery {
    pub fn matches(&self, item: &ScheduledRegionPinProfile) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledRegionPinProfile]) -> Vec<ScheduledRegionPinProfile> {
        let matched = collections::filter(items, |candidate: &ScheduledRegionPinProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_region_pin_profiles(
    store: Rc<dyn ScheduledRegionPinProfileStore>,
    query: &ScheduledRegionPinProfileQuery,
) -> Vec<ScheduledRegionPinProfile> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ScheduledRegionPinProfileStore> = new_memory_scheduled_region_pin_profile_store();
        let item = new_scheduled_region_pin_profile("code-943", "name-748");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
