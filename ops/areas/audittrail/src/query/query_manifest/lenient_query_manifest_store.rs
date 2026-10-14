use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient query manifests by code.
pub trait LenientQueryManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientQueryManifest>;
    fn save(&self, item: &LenientQueryManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientQueryManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientQueryManifestStore {
    pub items: RefCell<BTreeMap<String, LenientQueryManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientQueryManifestStore for MemoryLenientQueryManifestStore {
    fn find(&self, code: &str) -> Option<LenientQueryManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientQueryManifest) {
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

    fn list(&self) -> Vec<LenientQueryManifest> {
        let mut out: Vec<LenientQueryManifest> = Vec::new();
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

pub fn new_memory_lenient_query_manifest_store() -> Rc<dyn LenientQueryManifestStore> {
    Rc::new(MemoryLenientQueryManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientQueryManifestStore {
    pub inner: Rc<dyn LenientQueryManifestStore>,
    pub cache: RefCell<cache::TtlCache<LenientQueryManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientQueryManifestStore for CachedLenientQueryManifestStore {
    fn find(&self, code: &str) -> Option<LenientQueryManifest> {
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

    fn save(&self, item: &LenientQueryManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientQueryManifest> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_query_manifest_store(
    inner: Rc<dyn LenientQueryManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientQueryManifestStore> {
    Rc::new(CachedLenientQueryManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientQueryManifest>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient query manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryManifestQuery {
    pub segment: String,
    pub min_priority: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientQueryManifestQuery {
    pub fn matches(&self, item: &LenientQueryManifest) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.priority < self.min_priority {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientQueryManifest]) -> Vec<LenientQueryManifest> {
        let matched = collections::filter(items, |candidate: &LenientQueryManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_query_manifests(
    store: Rc<dyn LenientQueryManifestStore>,
    query: &LenientQueryManifestQuery,
) -> Vec<LenientQueryManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientQueryManifestStore> = new_memory_lenient_query_manifest_store();
        let item = new_lenient_query_manifest("code-344", "name-466");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
