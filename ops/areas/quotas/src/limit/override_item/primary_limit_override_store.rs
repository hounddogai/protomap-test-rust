use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary limit overrides by code.
pub trait PrimaryLimitOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryLimitOverride>;
    fn save(&self, item: &PrimaryLimitOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryLimitOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryLimitOverrideStore {
    pub items: RefCell<BTreeMap<String, PrimaryLimitOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryLimitOverrideStore for MemoryPrimaryLimitOverrideStore {
    fn find(&self, code: &str) -> Option<PrimaryLimitOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryLimitOverride) {
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

    fn list(&self) -> Vec<PrimaryLimitOverride> {
        let mut out: Vec<PrimaryLimitOverride> = Vec::new();
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

pub fn new_memory_primary_limit_override_store() -> Rc<dyn PrimaryLimitOverrideStore> {
    Rc::new(MemoryPrimaryLimitOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryLimitOverrideStore {
    pub inner: Rc<dyn PrimaryLimitOverrideStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryLimitOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryLimitOverrideStore for CachedPrimaryLimitOverrideStore {
    fn find(&self, code: &str) -> Option<PrimaryLimitOverride> {
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

    fn save(&self, item: &PrimaryLimitOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryLimitOverride> {
        self.inner.list()
    }
}

pub fn new_cached_primary_limit_override_store(
    inner: Rc<dyn PrimaryLimitOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryLimitOverrideStore> {
    Rc::new(CachedPrimaryLimitOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryLimitOverride>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary limit overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitOverrideQuery {
    pub locale: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryLimitOverrideQuery {
    pub fn matches(&self, item: &PrimaryLimitOverride) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.ratio < self.min_ratio {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryLimitOverride]) -> Vec<PrimaryLimitOverride> {
        let matched = collections::filter(items, |candidate: &PrimaryLimitOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_limit_overrides(
    store: Rc<dyn PrimaryLimitOverrideStore>,
    query: &PrimaryLimitOverrideQuery,
) -> Vec<PrimaryLimitOverride> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryLimitOverrideStore> = new_memory_primary_limit_override_store();
        let item = new_primary_limit_override("code-580", "name-532");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
