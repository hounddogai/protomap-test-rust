use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps adaptive rollback notices by code.
pub trait AdaptiveRollbackNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<AdaptiveRollbackNotice>;
    fn save(&self, item: &AdaptiveRollbackNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<AdaptiveRollbackNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAdaptiveRollbackNoticeStore {
    pub items: RefCell<BTreeMap<String, AdaptiveRollbackNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl AdaptiveRollbackNoticeStore for MemoryAdaptiveRollbackNoticeStore {
    fn find(&self, code: &str) -> Option<AdaptiveRollbackNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &AdaptiveRollbackNotice) {
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

    fn list(&self) -> Vec<AdaptiveRollbackNotice> {
        let mut out: Vec<AdaptiveRollbackNotice> = Vec::new();
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

pub fn new_memory_adaptive_rollback_notice_store() -> Rc<dyn AdaptiveRollbackNoticeStore> {
    Rc::new(MemoryAdaptiveRollbackNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedAdaptiveRollbackNoticeStore {
    pub inner: Rc<dyn AdaptiveRollbackNoticeStore>,
    pub cache: RefCell<cache::TtlCache<AdaptiveRollbackNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl AdaptiveRollbackNoticeStore for CachedAdaptiveRollbackNoticeStore {
    fn find(&self, code: &str) -> Option<AdaptiveRollbackNotice> {
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

    fn save(&self, item: &AdaptiveRollbackNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<AdaptiveRollbackNotice> {
        self.inner.list()
    }
}

pub fn new_cached_adaptive_rollback_notice_store(
    inner: Rc<dyn AdaptiveRollbackNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn AdaptiveRollbackNoticeStore> {
    Rc::new(CachedAdaptiveRollbackNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<AdaptiveRollbackNotice>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects adaptive rollback notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackNoticeQuery {
    pub source: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl AdaptiveRollbackNoticeQuery {
    pub fn matches(&self, item: &AdaptiveRollbackNotice) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[AdaptiveRollbackNotice]) -> Vec<AdaptiveRollbackNotice> {
        let matched = collections::filter(items, |candidate: &AdaptiveRollbackNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_adaptive_rollback_notices(
    store: Rc<dyn AdaptiveRollbackNoticeStore>,
    query: &AdaptiveRollbackNoticeQuery,
) -> Vec<AdaptiveRollbackNotice> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn AdaptiveRollbackNoticeStore> = new_memory_adaptive_rollback_notice_store();
        let item = new_adaptive_rollback_notice("code-130", "label-160");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
