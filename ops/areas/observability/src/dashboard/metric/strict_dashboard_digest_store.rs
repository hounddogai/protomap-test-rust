use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict dashboard digests by code.
pub trait StrictDashboardDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictDashboardDigest>;
    fn save(&self, item: &StrictDashboardDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictDashboardDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictDashboardDigestStore {
    pub items: RefCell<BTreeMap<String, StrictDashboardDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictDashboardDigestStore for MemoryStrictDashboardDigestStore {
    fn find(&self, code: &str) -> Option<StrictDashboardDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictDashboardDigest) {
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

    fn list(&self) -> Vec<StrictDashboardDigest> {
        let mut out: Vec<StrictDashboardDigest> = Vec::new();
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

pub fn new_memory_strict_dashboard_digest_store() -> Rc<dyn StrictDashboardDigestStore> {
    Rc::new(MemoryStrictDashboardDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictDashboardDigestStore {
    pub inner: Rc<dyn StrictDashboardDigestStore>,
    pub cache: RefCell<cache::TtlCache<StrictDashboardDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictDashboardDigestStore for CachedStrictDashboardDigestStore {
    fn find(&self, code: &str) -> Option<StrictDashboardDigest> {
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

    fn save(&self, item: &StrictDashboardDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictDashboardDigest> {
        self.inner.list()
    }
}

pub fn new_cached_strict_dashboard_digest_store(
    inner: Rc<dyn StrictDashboardDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictDashboardDigestStore> {
    Rc::new(CachedStrictDashboardDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictDashboardDigest>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict dashboard digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardDigestQuery {
    pub title: String,
    pub min_rate: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictDashboardDigestQuery {
    pub fn matches(&self, item: &StrictDashboardDigest) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[StrictDashboardDigest]) -> Vec<StrictDashboardDigest> {
        let matched = collections::filter(items, |candidate: &StrictDashboardDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_dashboard_digests(
    store: Rc<dyn StrictDashboardDigestStore>,
    query: &StrictDashboardDigestQuery,
) -> Vec<StrictDashboardDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn StrictDashboardDigestStore> = new_memory_strict_dashboard_digest_store();
        let item = new_strict_dashboard_digest("code-694", "title-582");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
