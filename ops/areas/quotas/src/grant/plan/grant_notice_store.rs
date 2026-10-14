use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps grant notices by code.
pub trait GrantNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<GrantNotice>;
    fn save(&self, item: &GrantNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<GrantNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryGrantNoticeStore {
    pub items: RefCell<BTreeMap<String, GrantNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl GrantNoticeStore for MemoryGrantNoticeStore {
    fn find(&self, code: &str) -> Option<GrantNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &GrantNotice) {
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

    fn list(&self) -> Vec<GrantNotice> {
        let mut out: Vec<GrantNotice> = Vec::new();
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

pub fn new_memory_grant_notice_store() -> Rc<dyn GrantNoticeStore> {
    Rc::new(MemoryGrantNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedGrantNoticeStore {
    pub inner: Rc<dyn GrantNoticeStore>,
    pub cache: RefCell<cache::TtlCache<GrantNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl GrantNoticeStore for CachedGrantNoticeStore {
    fn find(&self, code: &str) -> Option<GrantNotice> {
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

    fn save(&self, item: &GrantNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<GrantNotice> {
        self.inner.list()
    }
}

pub fn new_cached_grant_notice_store(
    inner: Rc<dyn GrantNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn GrantNoticeStore> {
    Rc::new(CachedGrantNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<GrantNotice>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects grant notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantNoticeQuery {
    pub namespace: String,
    pub min_ttl_seconds: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl GrantNoticeQuery {
    pub fn matches(&self, item: &GrantNotice) -> bool {
        if self.namespace != "" && (text::normalize(&item.namespace) != text::normalize(&self.namespace)) {
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

    pub fn apply(&self, items: &[GrantNotice]) -> Vec<GrantNotice> {
        let matched = collections::filter(items, |candidate: &GrantNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_grant_notices(store: Rc<dyn GrantNoticeStore>, query: &GrantNoticeQuery) -> Vec<GrantNotice> {
    query.apply(&store.list())
}
