use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient nonce preferences by code.
pub trait LenientNoncePreferenceStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientNoncePreference>;
    fn save(&self, item: &LenientNoncePreference);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientNoncePreference>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientNoncePreferenceStore {
    pub items: RefCell<BTreeMap<String, LenientNoncePreference>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientNoncePreferenceStore for MemoryLenientNoncePreferenceStore {
    fn find(&self, code: &str) -> Option<LenientNoncePreference> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientNoncePreference) {
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

    fn list(&self) -> Vec<LenientNoncePreference> {
        let mut out: Vec<LenientNoncePreference> = Vec::new();
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

pub fn new_memory_lenient_nonce_preference_store() -> Rc<dyn LenientNoncePreferenceStore> {
    Rc::new(MemoryLenientNoncePreferenceStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientNoncePreferenceStore {
    pub inner: Rc<dyn LenientNoncePreferenceStore>,
    pub cache: RefCell<cache::TtlCache<LenientNoncePreference>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientNoncePreferenceStore for CachedLenientNoncePreferenceStore {
    fn find(&self, code: &str) -> Option<LenientNoncePreference> {
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

    fn save(&self, item: &LenientNoncePreference) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientNoncePreference> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_nonce_preference_store(
    inner: Rc<dyn LenientNoncePreferenceStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientNoncePreferenceStore> {
    Rc::new(CachedLenientNoncePreferenceStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientNoncePreference>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient nonce preferences. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientNoncePreferenceQuery {
    pub source: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientNoncePreferenceQuery {
    pub fn matches(&self, item: &LenientNoncePreference) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientNoncePreference]) -> Vec<LenientNoncePreference> {
        let matched = collections::filter(items, |candidate: &LenientNoncePreference| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_nonce_preferences(
    store: Rc<dyn LenientNoncePreferenceStore>,
    query: &LenientNoncePreferenceQuery,
) -> Vec<LenientNoncePreference> {
    query.apply(&store.list())
}
