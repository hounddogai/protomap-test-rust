use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps primary identity scores by code.
pub trait PrimaryIdentityScoreStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PrimaryIdentityScore>;
    fn save(&self, item: &PrimaryIdentityScore);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PrimaryIdentityScore>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPrimaryIdentityScoreStore {
    pub items: RefCell<BTreeMap<String, PrimaryIdentityScore>>,
    pub order: RefCell<Vec<String>>,
}

impl PrimaryIdentityScoreStore for MemoryPrimaryIdentityScoreStore {
    fn find(&self, code: &str) -> Option<PrimaryIdentityScore> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PrimaryIdentityScore) {
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

    fn list(&self) -> Vec<PrimaryIdentityScore> {
        let mut out: Vec<PrimaryIdentityScore> = Vec::new();
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

pub fn new_memory_primary_identity_score_store() -> Rc<dyn PrimaryIdentityScoreStore> {
    Rc::new(MemoryPrimaryIdentityScoreStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPrimaryIdentityScoreStore {
    pub inner: Rc<dyn PrimaryIdentityScoreStore>,
    pub cache: RefCell<cache::TtlCache<PrimaryIdentityScore>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PrimaryIdentityScoreStore for CachedPrimaryIdentityScoreStore {
    fn find(&self, code: &str) -> Option<PrimaryIdentityScore> {
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

    fn save(&self, item: &PrimaryIdentityScore) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PrimaryIdentityScore> {
        self.inner.list()
    }
}

pub fn new_cached_primary_identity_score_store(
    inner: Rc<dyn PrimaryIdentityScoreStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PrimaryIdentityScoreStore> {
    Rc::new(CachedPrimaryIdentityScoreStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PrimaryIdentityScore>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects primary identity scores. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityScoreQuery {
    pub title: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PrimaryIdentityScoreQuery {
    pub fn matches(&self, item: &PrimaryIdentityScore) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PrimaryIdentityScore]) -> Vec<PrimaryIdentityScore> {
        let matched = collections::filter(items, |candidate: &PrimaryIdentityScore| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_primary_identity_scores(
    store: Rc<dyn PrimaryIdentityScoreStore>,
    query: &PrimaryIdentityScoreQuery,
) -> Vec<PrimaryIdentityScore> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn PrimaryIdentityScoreStore> = new_memory_primary_identity_score_store();
        let item = new_primary_identity_score("code-703", "title-644");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
