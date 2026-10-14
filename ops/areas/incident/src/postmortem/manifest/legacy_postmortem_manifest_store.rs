use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy postmortem manifests by code.
pub trait LegacyPostmortemManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyPostmortemManifest>;
    fn save(&self, item: &LegacyPostmortemManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyPostmortemManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyPostmortemManifestStore {
    pub items: RefCell<BTreeMap<String, LegacyPostmortemManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyPostmortemManifestStore for MemoryLegacyPostmortemManifestStore {
    fn find(&self, code: &str) -> Option<LegacyPostmortemManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyPostmortemManifest) {
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

    fn list(&self) -> Vec<LegacyPostmortemManifest> {
        let mut out: Vec<LegacyPostmortemManifest> = Vec::new();
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

pub fn new_memory_legacy_postmortem_manifest_store() -> Rc<dyn LegacyPostmortemManifestStore> {
    Rc::new(MemoryLegacyPostmortemManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyPostmortemManifestStore {
    pub inner: Rc<dyn LegacyPostmortemManifestStore>,
    pub cache: RefCell<cache::TtlCache<LegacyPostmortemManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyPostmortemManifestStore for CachedLegacyPostmortemManifestStore {
    fn find(&self, code: &str) -> Option<LegacyPostmortemManifest> {
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

    fn save(&self, item: &LegacyPostmortemManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyPostmortemManifest> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_postmortem_manifest_store(
    inner: Rc<dyn LegacyPostmortemManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyPostmortemManifestStore> {
    Rc::new(CachedLegacyPostmortemManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyPostmortemManifest>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy postmortem manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPostmortemManifestQuery {
    pub source: String,
    pub min_ceiling: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyPostmortemManifestQuery {
    pub fn matches(&self, item: &LegacyPostmortemManifest) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.ceiling < self.min_ceiling {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyPostmortemManifest]) -> Vec<LegacyPostmortemManifest> {
        let matched = collections::filter(items, |candidate: &LegacyPostmortemManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_postmortem_manifests(
    store: Rc<dyn LegacyPostmortemManifestStore>,
    query: &LegacyPostmortemManifestQuery,
) -> Vec<LegacyPostmortemManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LegacyPostmortemManifestStore> = new_memory_legacy_postmortem_manifest_store();
        let item = new_legacy_postmortem_manifest("code-712", "title-195");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
