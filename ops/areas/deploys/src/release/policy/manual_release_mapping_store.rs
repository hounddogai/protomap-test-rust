use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps manual release mappings by code.
pub trait ManualReleaseMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ManualReleaseMapping>;
    fn save(&self, item: &ManualReleaseMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ManualReleaseMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryManualReleaseMappingStore {
    pub items: RefCell<BTreeMap<String, ManualReleaseMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl ManualReleaseMappingStore for MemoryManualReleaseMappingStore {
    fn find(&self, code: &str) -> Option<ManualReleaseMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ManualReleaseMapping) {
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

    fn list(&self) -> Vec<ManualReleaseMapping> {
        let mut out: Vec<ManualReleaseMapping> = Vec::new();
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

pub fn new_memory_manual_release_mapping_store() -> Rc<dyn ManualReleaseMappingStore> {
    Rc::new(MemoryManualReleaseMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedManualReleaseMappingStore {
    pub inner: Rc<dyn ManualReleaseMappingStore>,
    pub cache: RefCell<cache::TtlCache<ManualReleaseMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ManualReleaseMappingStore for CachedManualReleaseMappingStore {
    fn find(&self, code: &str) -> Option<ManualReleaseMapping> {
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

    fn save(&self, item: &ManualReleaseMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ManualReleaseMapping> {
        self.inner.list()
    }
}

pub fn new_cached_manual_release_mapping_store(
    inner: Rc<dyn ManualReleaseMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ManualReleaseMappingStore> {
    Rc::new(CachedManualReleaseMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ManualReleaseMapping>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects manual release mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseMappingQuery {
    pub namespace: String,
    pub min_expires_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ManualReleaseMappingQuery {
    pub fn matches(&self, item: &ManualReleaseMapping) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.expires_at < self.min_expires_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ManualReleaseMapping]) -> Vec<ManualReleaseMapping> {
        let matched = collections::filter(items, |candidate: &ManualReleaseMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_manual_release_mappings(
    store: Rc<dyn ManualReleaseMappingStore>,
    query: &ManualReleaseMappingQuery,
) -> Vec<ManualReleaseMapping> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ManualReleaseMappingStore> = new_memory_manual_release_mapping_store();
        let item = new_manual_release_mapping("code-626", "title-806");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
