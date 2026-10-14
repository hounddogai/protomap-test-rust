use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps batched token notices by code.
pub trait BatchedTokenNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<BatchedTokenNotice>;
    fn save(&self, item: &BatchedTokenNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<BatchedTokenNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBatchedTokenNoticeStore {
    pub items: RefCell<BTreeMap<String, BatchedTokenNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl BatchedTokenNoticeStore for MemoryBatchedTokenNoticeStore {
    fn find(&self, code: &str) -> Option<BatchedTokenNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &BatchedTokenNotice) {
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

    fn list(&self) -> Vec<BatchedTokenNotice> {
        let mut out: Vec<BatchedTokenNotice> = Vec::new();
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

pub fn new_memory_batched_token_notice_store() -> Rc<dyn BatchedTokenNoticeStore> {
    Rc::new(MemoryBatchedTokenNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedBatchedTokenNoticeStore {
    pub inner: Rc<dyn BatchedTokenNoticeStore>,
    pub cache: RefCell<cache::TtlCache<BatchedTokenNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl BatchedTokenNoticeStore for CachedBatchedTokenNoticeStore {
    fn find(&self, code: &str) -> Option<BatchedTokenNotice> {
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

    fn save(&self, item: &BatchedTokenNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<BatchedTokenNotice> {
        self.inner.list()
    }
}

pub fn new_cached_batched_token_notice_store(
    inner: Rc<dyn BatchedTokenNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn BatchedTokenNoticeStore> {
    Rc::new(CachedBatchedTokenNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<BatchedTokenNotice>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects batched token notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenNoticeQuery {
    pub category: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl BatchedTokenNoticeQuery {
    pub fn matches(&self, item: &BatchedTokenNotice) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
            return false;
        }
        if item.ttl_seconds < self.min_ttl_seconds {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[BatchedTokenNotice]) -> Vec<BatchedTokenNotice> {
        let matched = collections::filter(items, |candidate: &BatchedTokenNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_batched_token_notices(
    store: Rc<dyn BatchedTokenNoticeStore>,
    query: &BatchedTokenNoticeQuery,
) -> Vec<BatchedTokenNotice> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn BatchedTokenNoticeStore> = new_memory_batched_token_notice_store();
        let item = new_batched_token_notice("code-534", "name-9");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
