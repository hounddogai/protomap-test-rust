use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered dashboard digests by code.
pub trait TieredDashboardDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredDashboardDigest>;
    fn save(&self, item: &TieredDashboardDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredDashboardDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredDashboardDigestStore {
    pub items: RefCell<BTreeMap<String, TieredDashboardDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredDashboardDigestStore for MemoryTieredDashboardDigestStore {
    fn find(&self, code: &str) -> Option<TieredDashboardDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredDashboardDigest) {
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

    fn list(&self) -> Vec<TieredDashboardDigest> {
        let mut out: Vec<TieredDashboardDigest> = Vec::new();
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

pub fn new_memory_tiered_dashboard_digest_store() -> Rc<dyn TieredDashboardDigestStore> {
    Rc::new(MemoryTieredDashboardDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredDashboardDigestStore {
    pub inner: Rc<dyn TieredDashboardDigestStore>,
    pub cache: RefCell<cache::TtlCache<TieredDashboardDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredDashboardDigestStore for CachedTieredDashboardDigestStore {
    fn find(&self, code: &str) -> Option<TieredDashboardDigest> {
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

    fn save(&self, item: &TieredDashboardDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredDashboardDigest> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_dashboard_digest_store(
    inner: Rc<dyn TieredDashboardDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredDashboardDigestStore> {
    Rc::new(CachedTieredDashboardDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredDashboardDigest>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered dashboard digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardDigestQuery {
    pub channel: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredDashboardDigestQuery {
    pub fn matches(&self, item: &TieredDashboardDigest) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
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

    pub fn apply(&self, items: &[TieredDashboardDigest]) -> Vec<TieredDashboardDigest> {
        let matched = collections::filter(items, |candidate: &TieredDashboardDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_dashboard_digests(
    store: Rc<dyn TieredDashboardDigestStore>,
    query: &TieredDashboardDigestQuery,
) -> Vec<TieredDashboardDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredDashboardDigestStore> = new_memory_tiered_dashboard_digest_store();
        let item = new_tiered_dashboard_digest("code-994", "title-58");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
