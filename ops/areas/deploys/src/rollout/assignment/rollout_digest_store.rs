use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rollout digests by code.
pub trait RolloutDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RolloutDigest>;
    fn save(&self, item: &RolloutDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RolloutDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRolloutDigestStore {
    pub items: RefCell<BTreeMap<String, RolloutDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl RolloutDigestStore for MemoryRolloutDigestStore {
    fn find(&self, code: &str) -> Option<RolloutDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RolloutDigest) {
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

    fn list(&self) -> Vec<RolloutDigest> {
        let mut out: Vec<RolloutDigest> = Vec::new();
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

pub fn new_memory_rollout_digest_store() -> Rc<dyn RolloutDigestStore> {
    Rc::new(MemoryRolloutDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRolloutDigestStore {
    pub inner: Rc<dyn RolloutDigestStore>,
    pub cache: RefCell<cache::TtlCache<RolloutDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RolloutDigestStore for CachedRolloutDigestStore {
    fn find(&self, code: &str) -> Option<RolloutDigest> {
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

    fn save(&self, item: &RolloutDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RolloutDigest> {
        self.inner.list()
    }
}

pub fn new_cached_rollout_digest_store(
    inner: Rc<dyn RolloutDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RolloutDigestStore> {
    Rc::new(CachedRolloutDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RolloutDigest>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rollout digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutDigestQuery {
    pub name: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RolloutDigestQuery {
    pub fn matches(&self, item: &RolloutDigest) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RolloutDigest]) -> Vec<RolloutDigest> {
        let matched = collections::filter(items, |candidate: &RolloutDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rollout_digests(store: Rc<dyn RolloutDigestStore>, query: &RolloutDigestQuery) -> Vec<RolloutDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RolloutDigestStore> = new_memory_rollout_digest_store();
        let item = new_rollout_digest("code-801", "name-818");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
