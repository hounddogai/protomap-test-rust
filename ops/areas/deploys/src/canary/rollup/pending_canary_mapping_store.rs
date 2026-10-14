use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps pending canary mappings by code.
pub trait PendingCanaryMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PendingCanaryMapping>;
    fn save(&self, item: &PendingCanaryMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PendingCanaryMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPendingCanaryMappingStore {
    pub items: RefCell<BTreeMap<String, PendingCanaryMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl PendingCanaryMappingStore for MemoryPendingCanaryMappingStore {
    fn find(&self, code: &str) -> Option<PendingCanaryMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PendingCanaryMapping) {
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

    fn list(&self) -> Vec<PendingCanaryMapping> {
        let mut out: Vec<PendingCanaryMapping> = Vec::new();
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

pub fn new_memory_pending_canary_mapping_store() -> Rc<dyn PendingCanaryMappingStore> {
    Rc::new(MemoryPendingCanaryMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPendingCanaryMappingStore {
    pub inner: Rc<dyn PendingCanaryMappingStore>,
    pub cache: RefCell<cache::TtlCache<PendingCanaryMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PendingCanaryMappingStore for CachedPendingCanaryMappingStore {
    fn find(&self, code: &str) -> Option<PendingCanaryMapping> {
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

    fn save(&self, item: &PendingCanaryMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PendingCanaryMapping> {
        self.inner.list()
    }
}

pub fn new_cached_pending_canary_mapping_store(
    inner: Rc<dyn PendingCanaryMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PendingCanaryMappingStore> {
    Rc::new(CachedPendingCanaryMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PendingCanaryMapping>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects pending canary mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryMappingQuery {
    pub country: String,
    pub min_starts_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PendingCanaryMappingQuery {
    pub fn matches(&self, item: &PendingCanaryMapping) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
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

    pub fn apply(&self, items: &[PendingCanaryMapping]) -> Vec<PendingCanaryMapping> {
        let matched = collections::filter(items, |candidate: &PendingCanaryMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_pending_canary_mappings(
    store: Rc<dyn PendingCanaryMappingStore>,
    query: &PendingCanaryMappingQuery,
) -> Vec<PendingCanaryMapping> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PendingCanaryMappingStore> = new_memory_pending_canary_mapping_store();
        let item = new_pending_canary_mapping("code-489", "title-190");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
