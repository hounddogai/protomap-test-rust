use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps zone profiles by code.
pub trait ZoneProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ZoneProfile>;
    fn save(&self, item: &ZoneProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ZoneProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryZoneProfileStore {
    pub items: RefCell<BTreeMap<String, ZoneProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl ZoneProfileStore for MemoryZoneProfileStore {
    fn find(&self, code: &str) -> Option<ZoneProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ZoneProfile) {
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

    fn list(&self) -> Vec<ZoneProfile> {
        let mut out: Vec<ZoneProfile> = Vec::new();
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

pub fn new_memory_zone_profile_store() -> Rc<dyn ZoneProfileStore> {
    Rc::new(MemoryZoneProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedZoneProfileStore {
    pub inner: Rc<dyn ZoneProfileStore>,
    pub cache: RefCell<cache::TtlCache<ZoneProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ZoneProfileStore for CachedZoneProfileStore {
    fn find(&self, code: &str) -> Option<ZoneProfile> {
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

    fn save(&self, item: &ZoneProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ZoneProfile> {
        self.inner.list()
    }
}

pub fn new_cached_zone_profile_store(
    inner: Rc<dyn ZoneProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ZoneProfileStore> {
    Rc::new(CachedZoneProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ZoneProfile>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects zone profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneProfileQuery {
    pub source: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ZoneProfileQuery {
    pub fn matches(&self, item: &ZoneProfile) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
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

    pub fn apply(&self, items: &[ZoneProfile]) -> Vec<ZoneProfile> {
        let matched = collections::filter(items, |candidate: &ZoneProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_zone_profiles(store: Rc<dyn ZoneProfileStore>, query: &ZoneProfileQuery) -> Vec<ZoneProfile> {
    query.apply(&store.list())
}
