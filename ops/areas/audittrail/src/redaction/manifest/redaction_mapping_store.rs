use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps redaction mappings by code.
pub trait RedactionMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RedactionMapping>;
    fn save(&self, item: &RedactionMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RedactionMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRedactionMappingStore {
    pub items: RefCell<BTreeMap<String, RedactionMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl RedactionMappingStore for MemoryRedactionMappingStore {
    fn find(&self, code: &str) -> Option<RedactionMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RedactionMapping) {
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

    fn list(&self) -> Vec<RedactionMapping> {
        let mut out: Vec<RedactionMapping> = Vec::new();
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

pub fn new_memory_redaction_mapping_store() -> Rc<dyn RedactionMappingStore> {
    Rc::new(MemoryRedactionMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRedactionMappingStore {
    pub inner: Rc<dyn RedactionMappingStore>,
    pub cache: RefCell<cache::TtlCache<RedactionMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RedactionMappingStore for CachedRedactionMappingStore {
    fn find(&self, code: &str) -> Option<RedactionMapping> {
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

    fn save(&self, item: &RedactionMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RedactionMapping> {
        self.inner.list()
    }
}

pub fn new_cached_redaction_mapping_store(
    inner: Rc<dyn RedactionMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RedactionMappingStore> {
    Rc::new(CachedRedactionMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RedactionMapping>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects redaction mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionMappingQuery {
    pub title: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RedactionMappingQuery {
    pub fn matches(&self, item: &RedactionMapping) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RedactionMapping]) -> Vec<RedactionMapping> {
        let matched = collections::filter(items, |candidate: &RedactionMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_redaction_mappings(
    store: Rc<dyn RedactionMappingStore>,
    query: &RedactionMappingQuery,
) -> Vec<RedactionMapping> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RedactionMappingStore> = new_memory_redaction_mapping_store();
        let item = new_redaction_mapping("code-384", "title-449");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
