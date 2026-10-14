use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps egress digests by code.
pub trait EgressDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<EgressDigest>;
    fn save(&self, item: &EgressDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<EgressDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEgressDigestStore {
    pub items: RefCell<BTreeMap<String, EgressDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl EgressDigestStore for MemoryEgressDigestStore {
    fn find(&self, code: &str) -> Option<EgressDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &EgressDigest) {
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

    fn list(&self) -> Vec<EgressDigest> {
        let mut out: Vec<EgressDigest> = Vec::new();
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

pub fn new_memory_egress_digest_store() -> Rc<dyn EgressDigestStore> {
    Rc::new(MemoryEgressDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedEgressDigestStore {
    pub inner: Rc<dyn EgressDigestStore>,
    pub cache: RefCell<cache::TtlCache<EgressDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl EgressDigestStore for CachedEgressDigestStore {
    fn find(&self, code: &str) -> Option<EgressDigest> {
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

    fn save(&self, item: &EgressDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<EgressDigest> {
        self.inner.list()
    }
}

pub fn new_cached_egress_digest_store(
    inner: Rc<dyn EgressDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn EgressDigestStore> {
    Rc::new(CachedEgressDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<EgressDigest>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects egress digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressDigestQuery {
    pub segment: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl EgressDigestQuery {
    pub fn matches(&self, item: &EgressDigest) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[EgressDigest]) -> Vec<EgressDigest> {
        let matched = collections::filter(items, |candidate: &EgressDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_egress_digests(store: Rc<dyn EgressDigestStore>, query: &EgressDigestQuery) -> Vec<EgressDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn EgressDigestStore> = new_memory_egress_digest_store();
        let item = new_egress_digest("code-371", "title-544");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
