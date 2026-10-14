use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps peering manifests by code.
pub trait PeeringManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PeeringManifest>;
    fn save(&self, item: &PeeringManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PeeringManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPeeringManifestStore {
    pub items: RefCell<BTreeMap<String, PeeringManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl PeeringManifestStore for MemoryPeeringManifestStore {
    fn find(&self, code: &str) -> Option<PeeringManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PeeringManifest) {
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

    fn list(&self) -> Vec<PeeringManifest> {
        let mut out: Vec<PeeringManifest> = Vec::new();
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

pub fn new_memory_peering_manifest_store() -> Rc<dyn PeeringManifestStore> {
    Rc::new(MemoryPeeringManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPeeringManifestStore {
    pub inner: Rc<dyn PeeringManifestStore>,
    pub cache: RefCell<cache::TtlCache<PeeringManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PeeringManifestStore for CachedPeeringManifestStore {
    fn find(&self, code: &str) -> Option<PeeringManifest> {
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

    fn save(&self, item: &PeeringManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PeeringManifest> {
        self.inner.list()
    }
}

pub fn new_cached_peering_manifest_store(
    inner: Rc<dyn PeeringManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PeeringManifestStore> {
    Rc::new(CachedPeeringManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PeeringManifest>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects peering manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringManifestQuery {
    pub locale: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PeeringManifestQuery {
    pub fn matches(&self, item: &PeeringManifest) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
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

    pub fn apply(&self, items: &[PeeringManifest]) -> Vec<PeeringManifest> {
        let matched = collections::filter(items, |candidate: &PeeringManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_peering_manifests(
    store: Rc<dyn PeeringManifestStore>,
    query: &PeeringManifestQuery,
) -> Vec<PeeringManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PeeringManifestStore> = new_memory_peering_manifest_store();
        let item = new_peering_manifest("code-361", "label-990");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
