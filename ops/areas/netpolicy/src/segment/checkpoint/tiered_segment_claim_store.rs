use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered segment claims by code.
pub trait TieredSegmentClaimStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredSegmentClaim>;
    fn save(&self, item: &TieredSegmentClaim);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredSegmentClaim>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredSegmentClaimStore {
    pub items: RefCell<BTreeMap<String, TieredSegmentClaim>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredSegmentClaimStore for MemoryTieredSegmentClaimStore {
    fn find(&self, code: &str) -> Option<TieredSegmentClaim> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredSegmentClaim) {
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

    fn list(&self) -> Vec<TieredSegmentClaim> {
        let mut out: Vec<TieredSegmentClaim> = Vec::new();
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

pub fn new_memory_tiered_segment_claim_store() -> Rc<dyn TieredSegmentClaimStore> {
    Rc::new(MemoryTieredSegmentClaimStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredSegmentClaimStore {
    pub inner: Rc<dyn TieredSegmentClaimStore>,
    pub cache: RefCell<cache::TtlCache<TieredSegmentClaim>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredSegmentClaimStore for CachedTieredSegmentClaimStore {
    fn find(&self, code: &str) -> Option<TieredSegmentClaim> {
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

    fn save(&self, item: &TieredSegmentClaim) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredSegmentClaim> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_segment_claim_store(
    inner: Rc<dyn TieredSegmentClaimStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredSegmentClaimStore> {
    Rc::new(CachedTieredSegmentClaimStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredSegmentClaim>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered segment claims. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentClaimQuery {
    pub locale: String,
    pub min_burst: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredSegmentClaimQuery {
    pub fn matches(&self, item: &TieredSegmentClaim) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.burst < self.min_burst {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[TieredSegmentClaim]) -> Vec<TieredSegmentClaim> {
        let matched = collections::filter(items, |candidate: &TieredSegmentClaim| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_segment_claims(
    store: Rc<dyn TieredSegmentClaimStore>,
    query: &TieredSegmentClaimQuery,
) -> Vec<TieredSegmentClaim> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredSegmentClaimStore> = new_memory_tiered_segment_claim_store();
        let item = new_tiered_segment_claim("code-227", "name-510");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
