use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped grant targets by code.
pub trait CappedGrantTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedGrantTarget>;
    fn save(&self, item: &CappedGrantTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedGrantTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedGrantTargetStore {
    pub items: RefCell<BTreeMap<String, CappedGrantTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedGrantTargetStore for MemoryCappedGrantTargetStore {
    fn find(&self, code: &str) -> Option<CappedGrantTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedGrantTarget) {
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

    fn list(&self) -> Vec<CappedGrantTarget> {
        let mut out: Vec<CappedGrantTarget> = Vec::new();
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

pub fn new_memory_capped_grant_target_store() -> Rc<dyn CappedGrantTargetStore> {
    Rc::new(MemoryCappedGrantTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedGrantTargetStore {
    pub inner: Rc<dyn CappedGrantTargetStore>,
    pub cache: RefCell<cache::TtlCache<CappedGrantTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedGrantTargetStore for CachedCappedGrantTargetStore {
    fn find(&self, code: &str) -> Option<CappedGrantTarget> {
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

    fn save(&self, item: &CappedGrantTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedGrantTarget> {
        self.inner.list()
    }
}

pub fn new_cached_capped_grant_target_store(
    inner: Rc<dyn CappedGrantTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedGrantTargetStore> {
    Rc::new(CachedCappedGrantTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedGrantTarget>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped grant targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedGrantTargetQuery {
    pub cluster: String,
    pub min_ratio: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedGrantTargetQuery {
    pub fn matches(&self, item: &CappedGrantTarget) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
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

    pub fn apply(&self, items: &[CappedGrantTarget]) -> Vec<CappedGrantTarget> {
        let matched = collections::filter(items, |candidate: &CappedGrantTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_grant_targets(
    store: Rc<dyn CappedGrantTargetStore>,
    query: &CappedGrantTargetQuery,
) -> Vec<CappedGrantTarget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedGrantTargetStore> = new_memory_capped_grant_target_store();
        let item = new_capped_grant_target("code-994", "name-258");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
