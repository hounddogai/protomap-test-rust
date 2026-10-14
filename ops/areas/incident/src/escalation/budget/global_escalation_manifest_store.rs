use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps global escalation manifests by code.
pub trait GlobalEscalationManifestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GlobalEscalationManifest>;
    fn save(&self, item: &GlobalEscalationManifest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GlobalEscalationManifest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGlobalEscalationManifestStore {
    pub items: RefCell<BTreeMap<String, GlobalEscalationManifest>>,
    pub order: RefCell<Vec<String>>,
}

impl GlobalEscalationManifestStore for MemoryGlobalEscalationManifestStore {
    fn find(&self, code: &str) -> Option<GlobalEscalationManifest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GlobalEscalationManifest) {
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

    fn list(&self) -> Vec<GlobalEscalationManifest> {
        let mut out: Vec<GlobalEscalationManifest> = Vec::new();
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

pub fn new_memory_global_escalation_manifest_store() -> Rc<dyn GlobalEscalationManifestStore> {
    Rc::new(MemoryGlobalEscalationManifestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGlobalEscalationManifestStore {
    pub inner: Rc<dyn GlobalEscalationManifestStore>,
    pub cache: RefCell<cache::TtlCache<GlobalEscalationManifest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GlobalEscalationManifestStore for CachedGlobalEscalationManifestStore {
    fn find(&self, code: &str) -> Option<GlobalEscalationManifest> {
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

    fn save(&self, item: &GlobalEscalationManifest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GlobalEscalationManifest> {
        self.inner.list()
    }
}

pub fn new_cached_global_escalation_manifest_store(
    inner: Rc<dyn GlobalEscalationManifestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GlobalEscalationManifestStore> {
    Rc::new(CachedGlobalEscalationManifestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GlobalEscalationManifest>(
            1024,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects global escalation manifests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEscalationManifestQuery {
    pub unit: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GlobalEscalationManifestQuery {
    pub fn matches(&self, item: &GlobalEscalationManifest) -> bool {
        if self.unit != "" && (text::normalize(&item.unit) != text::normalize(&self.unit)) {
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

    pub fn apply(&self, items: &[GlobalEscalationManifest]) -> Vec<GlobalEscalationManifest> {
        let matched = collections::filter(items, |candidate: &GlobalEscalationManifest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_global_escalation_manifests(
    store: Rc<dyn GlobalEscalationManifestStore>,
    query: &GlobalEscalationManifestQuery,
) -> Vec<GlobalEscalationManifest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn GlobalEscalationManifestStore> = new_memory_global_escalation_manifest_store();
        let item = new_global_escalation_manifest("code-234", "title-87");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
