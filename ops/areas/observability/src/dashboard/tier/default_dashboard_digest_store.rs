use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default dashboard digests by code.
pub trait DefaultDashboardDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultDashboardDigest>;
    fn save(&self, item: &DefaultDashboardDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultDashboardDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultDashboardDigestStore {
    pub items: RefCell<BTreeMap<String, DefaultDashboardDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultDashboardDigestStore for MemoryDefaultDashboardDigestStore {
    fn find(&self, code: &str) -> Option<DefaultDashboardDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultDashboardDigest) {
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

    fn list(&self) -> Vec<DefaultDashboardDigest> {
        let mut out: Vec<DefaultDashboardDigest> = Vec::new();
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

pub fn new_memory_default_dashboard_digest_store() -> Rc<dyn DefaultDashboardDigestStore> {
    Rc::new(MemoryDefaultDashboardDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultDashboardDigestStore {
    pub inner: Rc<dyn DefaultDashboardDigestStore>,
    pub cache: RefCell<cache::TtlCache<DefaultDashboardDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultDashboardDigestStore for CachedDefaultDashboardDigestStore {
    fn find(&self, code: &str) -> Option<DefaultDashboardDigest> {
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

    fn save(&self, item: &DefaultDashboardDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultDashboardDigest> {
        self.inner.list()
    }
}

pub fn new_cached_default_dashboard_digest_store(
    inner: Rc<dyn DefaultDashboardDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultDashboardDigestStore> {
    Rc::new(CachedDefaultDashboardDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultDashboardDigest>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default dashboard digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardDigestQuery {
    pub title: String,
    pub min_created_at: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultDashboardDigestQuery {
    pub fn matches(&self, item: &DefaultDashboardDigest) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.created_at < self.min_created_at {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultDashboardDigest]) -> Vec<DefaultDashboardDigest> {
        let matched = collections::filter(items, |candidate: &DefaultDashboardDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_dashboard_digests(
    store: Rc<dyn DefaultDashboardDigestStore>,
    query: &DefaultDashboardDigestQuery,
) -> Vec<DefaultDashboardDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DefaultDashboardDigestStore> = new_memory_default_dashboard_digest_store();
        let item = new_default_dashboard_digest("code-274", "title-858");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
