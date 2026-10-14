use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched credential mappings by code.
pub trait BatchedCredentialMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedCredentialMapping>;
    fn save(&self, item: &BatchedCredentialMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedCredentialMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedCredentialMappingStore {
    pub items: RefCell<BTreeMap<String, BatchedCredentialMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedCredentialMappingStore for MemoryBatchedCredentialMappingStore {
    fn find(&self, code: &str) -> Option<BatchedCredentialMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedCredentialMapping) {
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

    fn list(&self) -> Vec<BatchedCredentialMapping> {
        let mut out: Vec<BatchedCredentialMapping> = Vec::new();
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

pub fn new_memory_batched_credential_mapping_store() -> Rc<dyn BatchedCredentialMappingStore> {
    Rc::new(MemoryBatchedCredentialMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedCredentialMappingStore {
    pub inner: Rc<dyn BatchedCredentialMappingStore>,
    pub cache: RefCell<cache::TtlCache<BatchedCredentialMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedCredentialMappingStore for CachedBatchedCredentialMappingStore {
    fn find(&self, code: &str) -> Option<BatchedCredentialMapping> {
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

    fn save(&self, item: &BatchedCredentialMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedCredentialMapping> {
        self.inner.list()
    }
}

pub fn new_cached_batched_credential_mapping_store(
    inner: Rc<dyn BatchedCredentialMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedCredentialMappingStore> {
    Rc::new(CachedBatchedCredentialMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedCredentialMapping>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched credential mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialMappingQuery {
    pub cluster: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedCredentialMappingQuery {
    pub fn matches(&self, item: &BatchedCredentialMapping) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedCredentialMapping]) -> Vec<BatchedCredentialMapping> {
        let matched = collections::filter(items, |candidate: &BatchedCredentialMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_credential_mappings(
    store: Rc<dyn BatchedCredentialMappingStore>,
    query: &BatchedCredentialMappingQuery,
) -> Vec<BatchedCredentialMapping> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedCredentialMappingStore> = new_memory_batched_credential_mapping_store();
        let item = new_batched_credential_mapping("code-441", "label-584");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
