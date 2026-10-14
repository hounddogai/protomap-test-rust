use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps strict credential segments by code.
pub trait StrictCredentialSegmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<StrictCredentialSegment>;
    fn save(&self, item: &StrictCredentialSegment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<StrictCredentialSegment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryStrictCredentialSegmentStore {
    pub items: RefCell<BTreeMap<String, StrictCredentialSegment>>,
    pub order: RefCell<Vec<String>>,
}

impl StrictCredentialSegmentStore for MemoryStrictCredentialSegmentStore {
    fn find(&self, code: &str) -> Option<StrictCredentialSegment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &StrictCredentialSegment) {
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

    fn list(&self) -> Vec<StrictCredentialSegment> {
        let mut out: Vec<StrictCredentialSegment> = Vec::new();
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

pub fn new_memory_strict_credential_segment_store() -> Rc<dyn StrictCredentialSegmentStore> {
    Rc::new(MemoryStrictCredentialSegmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedStrictCredentialSegmentStore {
    pub inner: Rc<dyn StrictCredentialSegmentStore>,
    pub cache: RefCell<cache::TtlCache<StrictCredentialSegment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl StrictCredentialSegmentStore for CachedStrictCredentialSegmentStore {
    fn find(&self, code: &str) -> Option<StrictCredentialSegment> {
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

    fn save(&self, item: &StrictCredentialSegment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<StrictCredentialSegment> {
        self.inner.list()
    }
}

pub fn new_cached_strict_credential_segment_store(
    inner: Rc<dyn StrictCredentialSegmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn StrictCredentialSegmentStore> {
    Rc::new(CachedStrictCredentialSegmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<StrictCredentialSegment>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects strict credential segments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictCredentialSegmentQuery {
    pub country: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl StrictCredentialSegmentQuery {
    pub fn matches(&self, item: &StrictCredentialSegment) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.limit < self.min_limit {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[StrictCredentialSegment]) -> Vec<StrictCredentialSegment> {
        let matched = collections::filter(items, |candidate: &StrictCredentialSegment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_strict_credential_segments(
    store: Rc<dyn StrictCredentialSegmentStore>,
    query: &StrictCredentialSegmentQuery,
) -> Vec<StrictCredentialSegment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn StrictCredentialSegmentStore> = new_memory_strict_credential_segment_store();
        let item = new_strict_credential_segment("code-858", "label-158");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
