use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rollback notices by code.
pub trait RollbackNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollbackNotice>;
    fn save(&self, item: &RollbackNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollbackNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollbackNoticeStore {
    pub items: RefCell<BTreeMap<String, RollbackNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl RollbackNoticeStore for MemoryRollbackNoticeStore {
    fn find(&self, code: &str) -> Option<RollbackNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollbackNotice) {
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

    fn list(&self) -> Vec<RollbackNotice> {
        let mut out: Vec<RollbackNotice> = Vec::new();
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

pub fn new_memory_rollback_notice_store() -> Rc<dyn RollbackNoticeStore> {
    Rc::new(MemoryRollbackNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollbackNoticeStore {
    pub inner: Rc<dyn RollbackNoticeStore>,
    pub cache: RefCell<cache::TtlCache<RollbackNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollbackNoticeStore for CachedRollbackNoticeStore {
    fn find(&self, code: &str) -> Option<RollbackNotice> {
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

    fn save(&self, item: &RollbackNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollbackNotice> {
        self.inner.list()
    }
}

pub fn new_cached_rollback_notice_store(
    inner: Rc<dyn RollbackNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollbackNoticeStore> {
    Rc::new(CachedRollbackNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollbackNotice>(64, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rollback notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackNoticeQuery {
    pub scope: String,
    pub min_rotation_days: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollbackNoticeQuery {
    pub fn matches(&self, item: &RollbackNotice) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.rotation_days < self.min_rotation_days {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollbackNotice]) -> Vec<RollbackNotice> {
        let matched = collections::filter(items, |candidate: &RollbackNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rollback_notices(store: Rc<dyn RollbackNoticeStore>, query: &RollbackNoticeQuery) -> Vec<RollbackNotice> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RollbackNoticeStore> = new_memory_rollback_notice_store();
        let item = new_rollback_notice("code-333", "name-406");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
