use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary action item manifests by code.
pub trait PrimaryActionItemManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryActionItemManifest>;
    fn save(&self, item: &PrimaryActionItemManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryActionItemManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryActionItemManifestStore {
    pub items: RefCell<BTreeMap<String, PrimaryActionItemManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryActionItemManifestStore for MemoryPrimaryActionItemManifestStore {
    fn find(&self, code: &str) -> Option<PrimaryActionItemManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryActionItemManifest) {
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

    fn list(&self) -> Vec<PrimaryActionItemManifest> {
        let mut out: Vec<PrimaryActionItemManifest> = Vec::new();
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

pub fn new_memory_primary_action_item_manifest_store() -> Rc<dyn PrimaryActionItemManifestStore> {
    Rc::new(MemoryPrimaryActionItemManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryActionItemManifestStore {
    pub inner: Rc<dyn PrimaryActionItemManifestStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryActionItemManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryActionItemManifestStore for CachedPrimaryActionItemManifestStore {
    fn find(&self, code: &str) -> Option<PrimaryActionItemManifest> {
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

    fn save(&self, item: &PrimaryActionItemManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryActionItemManifest> {
        self.inner.list()
    }
}

pub fn new_cached_primary_action_item_manifest_store(
    inner: Rc<dyn PrimaryActionItemManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryActionItemManifestStore> {
    Rc::new(CachedPrimaryActionItemManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryActionItemManifest>(
            512,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary action item manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemManifestQuery {
    pub cluster: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryActionItemManifestQuery {
    pub fn matches(&self, item: &PrimaryActionItemManifest) -> bool {
        if self.cluster != "" && (text::normalize(&item.cluster) != text::normalize(&self.cluster)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryActionItemManifest]) -> Vec<PrimaryActionItemManifest> {
        let matched = collections::filter(items, |candidate: &PrimaryActionItemManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_action_item_manifests(
    store: Rc<dyn PrimaryActionItemManifestStore>,
    query: &PrimaryActionItemManifestQuery,
) -> Vec<PrimaryActionItemManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryActionItemManifestStore> = new_memory_primary_action_item_manifest_store();
        let item = new_primary_action_item_manifest("code-704", "label-675");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
