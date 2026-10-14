use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict grant targets by code.
pub trait StrictGrantTargetStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictGrantTarget>;
    fn save(&self, item: &StrictGrantTarget);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictGrantTarget>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictGrantTargetStore {
    pub items: RefCell<BTreeMap<String, StrictGrantTarget>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictGrantTargetStore for MemoryStrictGrantTargetStore {
    fn find(&self, code: &str) -> Option<StrictGrantTarget> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictGrantTarget) {
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

    fn list(&self) -> Vec<StrictGrantTarget> {
        let mut out: Vec<StrictGrantTarget> = Vec::new();
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

pub fn new_memory_strict_grant_target_store() -> Rc<dyn StrictGrantTargetStore> {
    Rc::new(MemoryStrictGrantTargetStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictGrantTargetStore {
    pub inner: Rc<dyn StrictGrantTargetStore>,
    pub cache: RefCell<cache::TtlCache<StrictGrantTarget>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictGrantTargetStore for CachedStrictGrantTargetStore {
    fn find(&self, code: &str) -> Option<StrictGrantTarget> {
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

    fn save(&self, item: &StrictGrantTarget) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictGrantTarget> {
        self.inner.list()
    }
}

pub fn new_cached_strict_grant_target_store(
    inner: Rc<dyn StrictGrantTargetStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictGrantTargetStore> {
    Rc::new(CachedStrictGrantTargetStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictGrantTarget>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict grant targets. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictGrantTargetQuery {
    pub label: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictGrantTargetQuery {
    pub fn matches(&self, item: &StrictGrantTarget) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.starts_at < self.min_starts_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictGrantTarget]) -> Vec<StrictGrantTarget> {
        let matched = collections::filter(items, |candidate: &StrictGrantTarget| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_grant_targets(
    store: Rc<dyn StrictGrantTargetStore>,
    query: &StrictGrantTargetQuery,
) -> Vec<StrictGrantTarget> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn StrictGrantTargetStore> = new_memory_strict_grant_target_store();
        let item = new_strict_grant_target("code-377", "label-89");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
