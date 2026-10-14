use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered nonce targets by code.
pub trait TieredNonceTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredNonceTarget>;
    fn save(&self, item: &TieredNonceTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredNonceTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredNonceTargetStore {
    pub items: RefCell<BTreeMap<String, TieredNonceTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredNonceTargetStore for MemoryTieredNonceTargetStore {
    fn find(&self, code: &str) -> Option<TieredNonceTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredNonceTarget) {
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

    fn list(&self) -> Vec<TieredNonceTarget> {
        let mut out: Vec<TieredNonceTarget> = Vec::new();
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

pub fn new_memory_tiered_nonce_target_store() -> Rc<dyn TieredNonceTargetStore> {
    Rc::new(MemoryTieredNonceTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredNonceTargetStore {
    pub inner: Rc<dyn TieredNonceTargetStore>,
    pub cache: RefCell<cache::TtlCache<TieredNonceTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredNonceTargetStore for CachedTieredNonceTargetStore {
    fn find(&self, code: &str) -> Option<TieredNonceTarget> {
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

    fn save(&self, item: &TieredNonceTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredNonceTarget> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_nonce_target_store(
    inner: Rc<dyn TieredNonceTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredNonceTargetStore> {
    Rc::new(CachedTieredNonceTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredNonceTarget>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered nonce targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceTargetQuery {
    pub title: String,
    pub min_attempts: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredNonceTargetQuery {
    pub fn matches(&self, item: &TieredNonceTarget) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.attempts < self.min_attempts {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredNonceTarget]) -> Vec<TieredNonceTarget> {
        let matched = collections::filter(items, |candidate: &TieredNonceTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_nonce_targets(
    store: Rc<dyn TieredNonceTargetStore>,
    query: &TieredNonceTargetQuery,
) -> Vec<TieredNonceTarget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredNonceTargetStore> = new_memory_tiered_nonce_target_store();
        let item = new_tiered_nonce_target("code-377", "title-384");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
