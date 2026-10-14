use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary policy rollups by code.
pub trait PrimaryPolicyRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryPolicyRollup>;
    fn save(&self, item: &PrimaryPolicyRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryPolicyRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryPolicyRollupStore {
    pub items: RefCell<BTreeMap<String, PrimaryPolicyRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryPolicyRollupStore for MemoryPrimaryPolicyRollupStore {
    fn find(&self, code: &str) -> Option<PrimaryPolicyRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryPolicyRollup) {
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

    fn list(&self) -> Vec<PrimaryPolicyRollup> {
        let mut out: Vec<PrimaryPolicyRollup> = Vec::new();
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

pub fn new_memory_primary_policy_rollup_store() -> Rc<dyn PrimaryPolicyRollupStore> {
    Rc::new(MemoryPrimaryPolicyRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryPolicyRollupStore {
    pub inner: Rc<dyn PrimaryPolicyRollupStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryPolicyRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryPolicyRollupStore for CachedPrimaryPolicyRollupStore {
    fn find(&self, code: &str) -> Option<PrimaryPolicyRollup> {
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

    fn save(&self, item: &PrimaryPolicyRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryPolicyRollup> {
        self.inner.list()
    }
}

pub fn new_cached_primary_policy_rollup_store(
    inner: Rc<dyn PrimaryPolicyRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryPolicyRollupStore> {
    Rc::new(CachedPrimaryPolicyRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryPolicyRollup>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary policy rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPolicyRollupQuery {
    pub source: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryPolicyRollupQuery {
    pub fn matches(&self, item: &PrimaryPolicyRollup) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryPolicyRollup]) -> Vec<PrimaryPolicyRollup> {
        let matched = collections::filter(items, |candidate: &PrimaryPolicyRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_policy_rollups(
    store: Rc<dyn PrimaryPolicyRollupStore>,
    query: &PrimaryPolicyRollupQuery,
) -> Vec<PrimaryPolicyRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryPolicyRollupStore> = new_memory_primary_policy_rollup_store();
        let item = new_primary_policy_rollup("code-58", "name-73");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
