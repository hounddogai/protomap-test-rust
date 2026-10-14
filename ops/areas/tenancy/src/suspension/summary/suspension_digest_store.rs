use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps suspension digests by code.
pub trait SuspensionDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SuspensionDigest>;
    fn save(&self, item: &SuspensionDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SuspensionDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySuspensionDigestStore {
    pub items: RefCell<BTreeMap<String, SuspensionDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl SuspensionDigestStore for MemorySuspensionDigestStore {
    fn find(&self, code: &str) -> Option<SuspensionDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SuspensionDigest) {
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

    fn list(&self) -> Vec<SuspensionDigest> {
        let mut out: Vec<SuspensionDigest> = Vec::new();
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

pub fn new_memory_suspension_digest_store() -> Rc<dyn SuspensionDigestStore> {
    Rc::new(MemorySuspensionDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSuspensionDigestStore {
    pub inner: Rc<dyn SuspensionDigestStore>,
    pub cache: RefCell<cache::TtlCache<SuspensionDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SuspensionDigestStore for CachedSuspensionDigestStore {
    fn find(&self, code: &str) -> Option<SuspensionDigest> {
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

    fn save(&self, item: &SuspensionDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SuspensionDigest> {
        self.inner.list()
    }
}

pub fn new_cached_suspension_digest_store(
    inner: Rc<dyn SuspensionDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SuspensionDigestStore> {
    Rc::new(CachedSuspensionDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SuspensionDigest>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects suspension digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionDigestQuery {
    pub source: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SuspensionDigestQuery {
    pub fn matches(&self, item: &SuspensionDigest) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.threshold < self.min_threshold {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SuspensionDigest]) -> Vec<SuspensionDigest> {
        let matched = collections::filter(items, |candidate: &SuspensionDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_suspension_digests(
    store: Rc<dyn SuspensionDigestStore>,
    query: &SuspensionDigestQuery,
) -> Vec<SuspensionDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn SuspensionDigestStore> = new_memory_suspension_digest_store();
        let item = new_suspension_digest("code-684", "name-867");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
