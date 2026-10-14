use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tenant records by code.
pub trait TenantRecordStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TenantRecord>;
    fn save(&self, item: &TenantRecord);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TenantRecord>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTenantRecordStore {
    pub items: RefCell<BTreeMap<String, TenantRecord>>,
    pub order: RefCell<Vec<String>>,
}

impl TenantRecordStore for MemoryTenantRecordStore {
    fn find(&self, code: &str) -> Option<TenantRecord> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TenantRecord) {
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

    fn list(&self) -> Vec<TenantRecord> {
        let mut out: Vec<TenantRecord> = Vec::new();
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

pub fn new_memory_tenant_record_store() -> Rc<dyn TenantRecordStore> {
    Rc::new(MemoryTenantRecordStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTenantRecordStore {
    pub inner: Rc<dyn TenantRecordStore>,
    pub cache: RefCell<cache::TtlCache<TenantRecord>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TenantRecordStore for CachedTenantRecordStore {
    fn find(&self, code: &str) -> Option<TenantRecord> {
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

    fn save(&self, item: &TenantRecord) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TenantRecord> {
        self.inner.list()
    }
}

pub fn new_cached_tenant_record_store(
    inner: Rc<dyn TenantRecordStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TenantRecordStore> {
    Rc::new(CachedTenantRecordStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TenantRecord>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tenant records. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantRecordQuery {
    pub namespace: String,
    pub min_budget: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TenantRecordQuery {
    pub fn matches(&self, item: &TenantRecord) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
            return false;
        }
        if item.budget < self.min_budget {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TenantRecord]) -> Vec<TenantRecord> {
        let matched = collections::filter(items, |candidate: &TenantRecord| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tenant_records(store: Rc<dyn TenantRecordStore>, query: &TenantRecordQuery) -> Vec<TenantRecord> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TenantRecordStore> = new_memory_tenant_record_store();
        let item = new_tenant_record("code-790", "name-475");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
