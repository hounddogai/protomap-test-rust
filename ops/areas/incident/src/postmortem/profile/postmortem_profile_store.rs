use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps postmortem profiles by code.
pub trait PostmortemProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PostmortemProfile>;
    fn save(&self, item: &PostmortemProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PostmortemProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPostmortemProfileStore {
    pub items: RefCell<BTreeMap<String, PostmortemProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl PostmortemProfileStore for MemoryPostmortemProfileStore {
    fn find(&self, code: &str) -> Option<PostmortemProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PostmortemProfile) {
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

    fn list(&self) -> Vec<PostmortemProfile> {
        let mut out: Vec<PostmortemProfile> = Vec::new();
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

pub fn new_memory_postmortem_profile_store() -> Rc<dyn PostmortemProfileStore> {
    Rc::new(MemoryPostmortemProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPostmortemProfileStore {
    pub inner: Rc<dyn PostmortemProfileStore>,
    pub cache: RefCell<cache::TtlCache<PostmortemProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PostmortemProfileStore for CachedPostmortemProfileStore {
    fn find(&self, code: &str) -> Option<PostmortemProfile> {
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

    fn save(&self, item: &PostmortemProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PostmortemProfile> {
        self.inner.list()
    }
}

pub fn new_cached_postmortem_profile_store(
    inner: Rc<dyn PostmortemProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PostmortemProfileStore> {
    Rc::new(CachedPostmortemProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PostmortemProfile>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects postmortem profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemProfileQuery {
    pub name: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PostmortemProfileQuery {
    pub fn matches(&self, item: &PostmortemProfile) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PostmortemProfile]) -> Vec<PostmortemProfile> {
        let matched = collections::filter(items, |candidate: &PostmortemProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_postmortem_profiles(
    store: Rc<dyn PostmortemProfileStore>,
    query: &PostmortemProfileQuery,
) -> Vec<PostmortemProfile> {
    query.apply(&store.list())
}
