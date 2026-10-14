use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary runbook mappings by code.
pub trait PrimaryRunbookMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryRunbookMapping>;
    fn save(&self, item: &PrimaryRunbookMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryRunbookMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryRunbookMappingStore {
    pub items: RefCell<BTreeMap<String, PrimaryRunbookMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryRunbookMappingStore for MemoryPrimaryRunbookMappingStore {
    fn find(&self, code: &str) -> Option<PrimaryRunbookMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryRunbookMapping) {
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

    fn list(&self) -> Vec<PrimaryRunbookMapping> {
        let mut out: Vec<PrimaryRunbookMapping> = Vec::new();
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

pub fn new_memory_primary_runbook_mapping_store() -> Rc<dyn PrimaryRunbookMappingStore> {
    Rc::new(MemoryPrimaryRunbookMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryRunbookMappingStore {
    pub inner: Rc<dyn PrimaryRunbookMappingStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryRunbookMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryRunbookMappingStore for CachedPrimaryRunbookMappingStore {
    fn find(&self, code: &str) -> Option<PrimaryRunbookMapping> {
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

    fn save(&self, item: &PrimaryRunbookMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryRunbookMapping> {
        self.inner.list()
    }
}

pub fn new_cached_primary_runbook_mapping_store(
    inner: Rc<dyn PrimaryRunbookMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryRunbookMappingStore> {
    Rc::new(CachedPrimaryRunbookMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryRunbookMapping>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary runbook mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRunbookMappingQuery {
    pub locale: String,
    pub min_port: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryRunbookMappingQuery {
    pub fn matches(&self, item: &PrimaryRunbookMapping) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.port < self.min_port {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryRunbookMapping]) -> Vec<PrimaryRunbookMapping> {
        let matched = collections::filter(items, |candidate: &PrimaryRunbookMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_runbook_mappings(
    store: Rc<dyn PrimaryRunbookMappingStore>,
    query: &PrimaryRunbookMappingQuery,
) -> Vec<PrimaryRunbookMapping> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryRunbookMappingStore> = new_memory_primary_runbook_mapping_store();
        let item = new_primary_runbook_mapping("code-433", "label-805");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
