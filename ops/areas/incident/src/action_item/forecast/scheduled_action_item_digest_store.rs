use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps scheduled action item digests by code.
pub trait ScheduledActionItemDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<ScheduledActionItemDigest>;
    fn save(&self, item: &ScheduledActionItemDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<ScheduledActionItemDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScheduledActionItemDigestStore {
    pub items: RefCell<BTreeMap<String, ScheduledActionItemDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl ScheduledActionItemDigestStore for MemoryScheduledActionItemDigestStore {
    fn find(&self, code: &str) -> Option<ScheduledActionItemDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &ScheduledActionItemDigest) {
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

    fn list(&self) -> Vec<ScheduledActionItemDigest> {
        let mut out: Vec<ScheduledActionItemDigest> = Vec::new();
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

pub fn new_memory_scheduled_action_item_digest_store() -> Rc<dyn ScheduledActionItemDigestStore> {
    Rc::new(MemoryScheduledActionItemDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedScheduledActionItemDigestStore {
    pub inner: Rc<dyn ScheduledActionItemDigestStore>,
    pub cache: RefCell<cache::TtlCache<ScheduledActionItemDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl ScheduledActionItemDigestStore for CachedScheduledActionItemDigestStore {
    fn find(&self, code: &str) -> Option<ScheduledActionItemDigest> {
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

    fn save(&self, item: &ScheduledActionItemDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<ScheduledActionItemDigest> {
        self.inner.list()
    }
}

pub fn new_cached_scheduled_action_item_digest_store(
    inner: Rc<dyn ScheduledActionItemDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn ScheduledActionItemDigestStore> {
    Rc::new(CachedScheduledActionItemDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<ScheduledActionItemDigest>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects scheduled action item digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledActionItemDigestQuery {
    pub label: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl ScheduledActionItemDigestQuery {
    pub fn matches(&self, item: &ScheduledActionItemDigest) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[ScheduledActionItemDigest]) -> Vec<ScheduledActionItemDigest> {
        let matched = collections::filter(items, |candidate: &ScheduledActionItemDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_scheduled_action_item_digests(
    store: Rc<dyn ScheduledActionItemDigestStore>,
    query: &ScheduledActionItemDigestQuery,
) -> Vec<ScheduledActionItemDigest> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn ScheduledActionItemDigestStore> = new_memory_scheduled_action_item_digest_store();
        let item = new_scheduled_action_item_digest("code-350", "label-216");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
