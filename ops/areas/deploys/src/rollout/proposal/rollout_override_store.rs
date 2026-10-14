use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rollout overrides by code.
pub trait RolloutOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RolloutOverride>;
    fn save(&self, item: &RolloutOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RolloutOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRolloutOverrideStore {
    pub items: RefCell<BTreeMap<String, RolloutOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl RolloutOverrideStore for MemoryRolloutOverrideStore {
    fn find(&self, code: &str) -> Option<RolloutOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RolloutOverride) {
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

    fn list(&self) -> Vec<RolloutOverride> {
        let mut out: Vec<RolloutOverride> = Vec::new();
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

pub fn new_memory_rollout_override_store() -> Rc<dyn RolloutOverrideStore> {
    Rc::new(MemoryRolloutOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRolloutOverrideStore {
    pub inner: Rc<dyn RolloutOverrideStore>,
    pub cache: RefCell<cache::TtlCache<RolloutOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RolloutOverrideStore for CachedRolloutOverrideStore {
    fn find(&self, code: &str) -> Option<RolloutOverride> {
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

    fn save(&self, item: &RolloutOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RolloutOverride> {
        self.inner.list()
    }
}

pub fn new_cached_rollout_override_store(
    inner: Rc<dyn RolloutOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RolloutOverrideStore> {
    Rc::new(CachedRolloutOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RolloutOverride>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rollout overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutOverrideQuery {
    pub label: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RolloutOverrideQuery {
    pub fn matches(&self, item: &RolloutOverride) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.ceiling < self.min_ceiling {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RolloutOverride]) -> Vec<RolloutOverride> {
        let matched = collections::filter(items, |candidate: &RolloutOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rollout_overrides(
    store: Rc<dyn RolloutOverrideStore>,
    query: &RolloutOverrideQuery,
) -> Vec<RolloutOverride> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RolloutOverrideStore> = new_memory_rollout_override_store();
        let item = new_rollout_override("code-772", "label-45");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
