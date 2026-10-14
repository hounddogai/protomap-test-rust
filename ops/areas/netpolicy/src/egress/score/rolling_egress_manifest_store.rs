use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling egress manifests by code.
pub trait RollingEgressManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingEgressManifest>;
    fn save(&self, item: &RollingEgressManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingEgressManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingEgressManifestStore {
    pub items: RefCell<BTreeMap<String, RollingEgressManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingEgressManifestStore for MemoryRollingEgressManifestStore {
    fn find(&self, code: &str) -> Option<RollingEgressManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingEgressManifest) {
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

    fn list(&self) -> Vec<RollingEgressManifest> {
        let mut out: Vec<RollingEgressManifest> = Vec::new();
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

pub fn new_memory_rolling_egress_manifest_store() -> Rc<dyn RollingEgressManifestStore> {
    Rc::new(MemoryRollingEgressManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingEgressManifestStore {
    pub inner: Rc<dyn RollingEgressManifestStore>,
    pub cache: RefCell<cache::TtlCache<RollingEgressManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingEgressManifestStore for CachedRollingEgressManifestStore {
    fn find(&self, code: &str) -> Option<RollingEgressManifest> {
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

    fn save(&self, item: &RollingEgressManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingEgressManifest> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_egress_manifest_store(
    inner: Rc<dyn RollingEgressManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingEgressManifestStore> {
    Rc::new(CachedRollingEgressManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingEgressManifest>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling egress manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressManifestQuery {
    pub country: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingEgressManifestQuery {
    pub fn matches(&self, item: &RollingEgressManifest) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingEgressManifest]) -> Vec<RollingEgressManifest> {
        let matched = collections::filter(items, |candidate: &RollingEgressManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_egress_manifests(
    store: Rc<dyn RollingEgressManifestStore>,
    query: &RollingEgressManifestQuery,
) -> Vec<RollingEgressManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollingEgressManifestStore> = new_memory_rolling_egress_manifest_store();
        let item = new_rolling_egress_manifest("code-55", "title-412");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
