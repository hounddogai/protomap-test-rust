use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling data export mappings by code.
pub trait RollingDataExportMappingStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingDataExportMapping>;
    fn save(&self, item: &RollingDataExportMapping);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingDataExportMapping>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingDataExportMappingStore {
    pub items: RefCell<BTreeMap<String, RollingDataExportMapping>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingDataExportMappingStore for MemoryRollingDataExportMappingStore {
    fn find(&self, code: &str) -> Option<RollingDataExportMapping> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingDataExportMapping) {
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

    fn list(&self) -> Vec<RollingDataExportMapping> {
        let mut out: Vec<RollingDataExportMapping> = Vec::new();
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

pub fn new_memory_rolling_data_export_mapping_store() -> Rc<dyn RollingDataExportMappingStore> {
    Rc::new(MemoryRollingDataExportMappingStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingDataExportMappingStore {
    pub inner: Rc<dyn RollingDataExportMappingStore>,
    pub cache: RefCell<cache::TtlCache<RollingDataExportMapping>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingDataExportMappingStore for CachedRollingDataExportMappingStore {
    fn find(&self, code: &str) -> Option<RollingDataExportMapping> {
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

    fn save(&self, item: &RollingDataExportMapping) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingDataExportMapping> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_data_export_mapping_store(
    inner: Rc<dyn RollingDataExportMappingStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingDataExportMappingStore> {
    Rc::new(CachedRollingDataExportMappingStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingDataExportMapping>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling data export mappings. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingDataExportMappingQuery {
    pub name: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingDataExportMappingQuery {
    pub fn matches(&self, item: &RollingDataExportMapping) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingDataExportMapping]) -> Vec<RollingDataExportMapping> {
        let matched = collections::filter(items, |candidate: &RollingDataExportMapping| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_data_export_mappings(
    store: Rc<dyn RollingDataExportMappingStore>,
    query: &RollingDataExportMappingQuery,
) -> Vec<RollingDataExportMapping> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingDataExportMappingStore> = new_memory_rolling_data_export_mapping_store();
        let item = new_rolling_data_export_mapping("code-187", "name-731");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
