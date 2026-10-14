use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional credential rollups by code.
pub trait RegionalCredentialRollupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalCredentialRollup>;
    fn save(&self, item: &RegionalCredentialRollup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalCredentialRollup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalCredentialRollupStore {
    pub items: RefCell<BTreeMap<String, RegionalCredentialRollup>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalCredentialRollupStore for MemoryRegionalCredentialRollupStore {
    fn find(&self, code: &str) -> Option<RegionalCredentialRollup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalCredentialRollup) {
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

    fn list(&self) -> Vec<RegionalCredentialRollup> {
        let mut out: Vec<RegionalCredentialRollup> = Vec::new();
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

pub fn new_memory_regional_credential_rollup_store() -> Rc<dyn RegionalCredentialRollupStore> {
    Rc::new(MemoryRegionalCredentialRollupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalCredentialRollupStore {
    pub inner: Rc<dyn RegionalCredentialRollupStore>,
    pub cache: RefCell<cache::TtlCache<RegionalCredentialRollup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalCredentialRollupStore for CachedRegionalCredentialRollupStore {
    fn find(&self, code: &str) -> Option<RegionalCredentialRollup> {
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

    fn save(&self, item: &RegionalCredentialRollup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalCredentialRollup> {
        self.inner.list()
    }
}

pub fn new_cached_regional_credential_rollup_store(
    inner: Rc<dyn RegionalCredentialRollupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalCredentialRollupStore> {
    Rc::new(CachedRegionalCredentialRollupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalCredentialRollup>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional credential rollups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCredentialRollupQuery {
    pub label: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalCredentialRollupQuery {
    pub fn matches(&self, item: &RegionalCredentialRollup) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.rate < self.min_rate {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalCredentialRollup]) -> Vec<RegionalCredentialRollup> {
        let matched = collections::filter(items, |candidate: &RegionalCredentialRollup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_credential_rollups(
    store: Rc<dyn RegionalCredentialRollupStore>,
    query: &RegionalCredentialRollupQuery,
) -> Vec<RegionalCredentialRollup> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalCredentialRollupStore> = new_memory_regional_credential_rollup_store();
        let item = new_regional_credential_rollup("code-381", "label-205");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
