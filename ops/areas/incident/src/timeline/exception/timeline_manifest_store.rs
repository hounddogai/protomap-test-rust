use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps timeline manifests by code.
pub trait TimelineManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TimelineManifest>;
    fn save(&self, item: &TimelineManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TimelineManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTimelineManifestStore {
    pub items: RefCell<BTreeMap<String, TimelineManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl TimelineManifestStore for MemoryTimelineManifestStore {
    fn find(&self, code: &str) -> Option<TimelineManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TimelineManifest) {
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

    fn list(&self) -> Vec<TimelineManifest> {
        let mut out: Vec<TimelineManifest> = Vec::new();
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

pub fn new_memory_timeline_manifest_store() -> Rc<dyn TimelineManifestStore> {
    Rc::new(MemoryTimelineManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTimelineManifestStore {
    pub inner: Rc<dyn TimelineManifestStore>,
    pub cache: RefCell<cache::TtlCache<TimelineManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TimelineManifestStore for CachedTimelineManifestStore {
    fn find(&self, code: &str) -> Option<TimelineManifest> {
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

    fn save(&self, item: &TimelineManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TimelineManifest> {
        self.inner.list()
    }
}

pub fn new_cached_timeline_manifest_store(
    inner: Rc<dyn TimelineManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TimelineManifestStore> {
    Rc::new(CachedTimelineManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TimelineManifest>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects timeline manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineManifestQuery {
    pub unit: String,
    pub min_maximum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TimelineManifestQuery {
    pub fn matches(&self, item: &TimelineManifest) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
            return false;
        }
        if item.maximum < self.min_maximum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TimelineManifest]) -> Vec<TimelineManifest> {
        let matched = collections::filter(items, |candidate: &TimelineManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_timeline_manifests(
    store: Rc<dyn TimelineManifestStore>,
    query: &TimelineManifestQuery,
) -> Vec<TimelineManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TimelineManifestStore> = new_memory_timeline_manifest_store();
        let item = new_timeline_manifest("code-862", "label-504");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
