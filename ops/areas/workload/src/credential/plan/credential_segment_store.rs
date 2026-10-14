use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps credential segments by code.
pub trait CredentialSegmentStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CredentialSegment>;
    fn save(&self, item: &CredentialSegment);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CredentialSegment>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCredentialSegmentStore {
    pub items: RefCell<BTreeMap<String, CredentialSegment>>,
    pub order: RefCell<Vec<String>>,
}

impl CredentialSegmentStore for MemoryCredentialSegmentStore {
    fn find(&self, code: &str) -> Option<CredentialSegment> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CredentialSegment) {
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

    fn list(&self) -> Vec<CredentialSegment> {
        let mut out: Vec<CredentialSegment> = Vec::new();
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

pub fn new_memory_credential_segment_store() -> Rc<dyn CredentialSegmentStore> {
    Rc::new(MemoryCredentialSegmentStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCredentialSegmentStore {
    pub inner: Rc<dyn CredentialSegmentStore>,
    pub cache: RefCell<cache::TtlCache<CredentialSegment>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CredentialSegmentStore for CachedCredentialSegmentStore {
    fn find(&self, code: &str) -> Option<CredentialSegment> {
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

    fn save(&self, item: &CredentialSegment) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CredentialSegment> {
        self.inner.list()
    }
}

pub fn new_cached_credential_segment_store(
    inner: Rc<dyn CredentialSegmentStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CredentialSegmentStore> {
    Rc::new(CachedCredentialSegmentStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CredentialSegment>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects credential segments. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSegmentQuery {
    pub region: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CredentialSegmentQuery {
    pub fn matches(&self, item: &CredentialSegment) -> bool {
        if self.region != "" && (text::normalize(&item.region) != text::normalize(&self.region)) {
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

    pub fn apply(&self, items: &[CredentialSegment]) -> Vec<CredentialSegment> {
        let matched = collections::filter(items, |candidate: &CredentialSegment| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_credential_segments(
    store: Rc<dyn CredentialSegmentStore>,
    query: &CredentialSegmentQuery,
) -> Vec<CredentialSegment> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CredentialSegmentStore> = new_memory_credential_segment_store();
        let item = new_credential_segment("code-556", "label-108");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
