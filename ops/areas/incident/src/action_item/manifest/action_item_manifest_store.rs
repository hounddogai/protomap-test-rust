use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps action item manifests by code.
pub trait ActionItemManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ActionItemManifest>;
    fn save(&self, item: &ActionItemManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ActionItemManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryActionItemManifestStore {
    pub items: RefCell<BTreeMap<String, ActionItemManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl ActionItemManifestStore for MemoryActionItemManifestStore {
    fn find(&self, code: &str) -> Option<ActionItemManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ActionItemManifest) {
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

    fn list(&self) -> Vec<ActionItemManifest> {
        let mut out: Vec<ActionItemManifest> = Vec::new();
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

pub fn new_memory_action_item_manifest_store() -> Rc<dyn ActionItemManifestStore> {
    Rc::new(MemoryActionItemManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedActionItemManifestStore {
    pub inner: Rc<dyn ActionItemManifestStore>,
    pub cache: RefCell<cache::TtlCache<ActionItemManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ActionItemManifestStore for CachedActionItemManifestStore {
    fn find(&self, code: &str) -> Option<ActionItemManifest> {
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

    fn save(&self, item: &ActionItemManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ActionItemManifest> {
        self.inner.list()
    }
}

pub fn new_cached_action_item_manifest_store(
    inner: Rc<dyn ActionItemManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ActionItemManifestStore> {
    Rc::new(CachedActionItemManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ActionItemManifest>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects action item manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemManifestQuery {
    pub channel: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ActionItemManifestQuery {
    pub fn matches(&self, item: &ActionItemManifest) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
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

    pub fn apply(&self, items: &[ActionItemManifest]) -> Vec<ActionItemManifest> {
        let matched = collections::filter(items, |candidate: &ActionItemManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_action_item_manifests(
    store: Rc<dyn ActionItemManifestStore>,
    query: &ActionItemManifestQuery,
) -> Vec<ActionItemManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ActionItemManifestStore> = new_memory_action_item_manifest_store();
        let item = new_action_item_manifest("code-376", "title-618");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
