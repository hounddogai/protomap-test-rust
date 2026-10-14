use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered query manifests by code.
pub trait TieredQueryManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredQueryManifest>;
    fn save(&self, item: &TieredQueryManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredQueryManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredQueryManifestStore {
    pub items: RefCell<BTreeMap<String, TieredQueryManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredQueryManifestStore for MemoryTieredQueryManifestStore {
    fn find(&self, code: &str) -> Option<TieredQueryManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredQueryManifest) {
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

    fn list(&self) -> Vec<TieredQueryManifest> {
        let mut out: Vec<TieredQueryManifest> = Vec::new();
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

pub fn new_memory_tiered_query_manifest_store() -> Rc<dyn TieredQueryManifestStore> {
    Rc::new(MemoryTieredQueryManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredQueryManifestStore {
    pub inner: Rc<dyn TieredQueryManifestStore>,
    pub cache: RefCell<cache::TtlCache<TieredQueryManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredQueryManifestStore for CachedTieredQueryManifestStore {
    fn find(&self, code: &str) -> Option<TieredQueryManifest> {
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

    fn save(&self, item: &TieredQueryManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredQueryManifest> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_query_manifest_store(
    inner: Rc<dyn TieredQueryManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredQueryManifestStore> {
    Rc::new(CachedTieredQueryManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredQueryManifest>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered query manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryManifestQuery {
    pub segment: String,
    pub min_grace_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredQueryManifestQuery {
    pub fn matches(&self, item: &TieredQueryManifest) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.grace_days < self.min_grace_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredQueryManifest]) -> Vec<TieredQueryManifest> {
        let matched = collections::filter(items, |candidate: &TieredQueryManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_query_manifests(
    store: Rc<dyn TieredQueryManifestStore>,
    query: &TieredQueryManifestQuery,
) -> Vec<TieredQueryManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredQueryManifestStore> = new_memory_tiered_query_manifest_store();
        let item = new_tiered_query_manifest("code-659", "title-183");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
