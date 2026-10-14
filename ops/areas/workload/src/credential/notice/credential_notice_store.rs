use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps credential notices by code.
pub trait CredentialNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CredentialNotice>;
    fn save(&self, item: &CredentialNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CredentialNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCredentialNoticeStore {
    pub items: RefCell<BTreeMap<String, CredentialNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl CredentialNoticeStore for MemoryCredentialNoticeStore {
    fn find(&self, code: &str) -> Option<CredentialNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CredentialNotice) {
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

    fn list(&self) -> Vec<CredentialNotice> {
        let mut out: Vec<CredentialNotice> = Vec::new();
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

pub fn new_memory_credential_notice_store() -> Rc<dyn CredentialNoticeStore> {
    Rc::new(MemoryCredentialNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCredentialNoticeStore {
    pub inner: Rc<dyn CredentialNoticeStore>,
    pub cache: RefCell<cache::TtlCache<CredentialNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CredentialNoticeStore for CachedCredentialNoticeStore {
    fn find(&self, code: &str) -> Option<CredentialNotice> {
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

    fn save(&self, item: &CredentialNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CredentialNotice> {
        self.inner.list()
    }
}

pub fn new_cached_credential_notice_store(
    inner: Rc<dyn CredentialNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CredentialNoticeStore> {
    Rc::new(CachedCredentialNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CredentialNotice>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects credential notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialNoticeQuery {
    pub segment: String,
    pub min_sequence: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CredentialNoticeQuery {
    pub fn matches(&self, item: &CredentialNotice) -> bool {
        if self.segment != "" && (text::normalize(&item.segment) != text::normalize(&self.segment)) {
            return false;
        }
        if item.sequence < self.min_sequence {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[CredentialNotice]) -> Vec<CredentialNotice> {
        let matched = collections::filter(items, |candidate: &CredentialNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_credential_notices(
    store: Rc<dyn CredentialNoticeStore>,
    query: &CredentialNoticeQuery,
) -> Vec<CredentialNotice> {
    query.apply(&store.list())
}
