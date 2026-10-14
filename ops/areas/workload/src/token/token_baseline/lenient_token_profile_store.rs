use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient token profiles by code.
pub trait LenientTokenProfileStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientTokenProfile>;
    fn save(&self, item: &LenientTokenProfile);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientTokenProfile>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientTokenProfileStore {
    pub items: RefCell<BTreeMap<String, LenientTokenProfile>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientTokenProfileStore for MemoryLenientTokenProfileStore {
    fn find(&self, code: &str) -> Option<LenientTokenProfile> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientTokenProfile) {
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

    fn list(&self) -> Vec<LenientTokenProfile> {
        let mut out: Vec<LenientTokenProfile> = Vec::new();
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

pub fn new_memory_lenient_token_profile_store() -> Rc<dyn LenientTokenProfileStore> {
    Rc::new(MemoryLenientTokenProfileStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientTokenProfileStore {
    pub inner: Rc<dyn LenientTokenProfileStore>,
    pub cache: RefCell<cache::TtlCache<LenientTokenProfile>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientTokenProfileStore for CachedLenientTokenProfileStore {
    fn find(&self, code: &str) -> Option<LenientTokenProfile> {
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

    fn save(&self, item: &LenientTokenProfile) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientTokenProfile> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_token_profile_store(
    inner: Rc<dyn LenientTokenProfileStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientTokenProfileStore> {
    Rc::new(CachedLenientTokenProfileStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientTokenProfile>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient token profiles. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTokenProfileQuery {
    pub scope: String,
    pub min_cost: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientTokenProfileQuery {
    pub fn matches(&self, item: &LenientTokenProfile) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.cost < self.min_cost {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientTokenProfile]) -> Vec<LenientTokenProfile> {
        let matched = collections::filter(items, |candidate: &LenientTokenProfile| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_token_profiles(
    store: Rc<dyn LenientTokenProfileStore>,
    query: &LenientTokenProfileQuery,
) -> Vec<LenientTokenProfile> {
    query.apply(&store.list())
}
