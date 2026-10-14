use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient credential notices by code.
pub trait LenientCredentialNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientCredentialNotice>;
    fn save(&self, item: &LenientCredentialNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientCredentialNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientCredentialNoticeStore {
    pub items: RefCell<BTreeMap<String, LenientCredentialNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientCredentialNoticeStore for MemoryLenientCredentialNoticeStore {
    fn find(&self, code: &str) -> Option<LenientCredentialNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientCredentialNotice) {
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

    fn list(&self) -> Vec<LenientCredentialNotice> {
        let mut out: Vec<LenientCredentialNotice> = Vec::new();
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

pub fn new_memory_lenient_credential_notice_store() -> Rc<dyn LenientCredentialNoticeStore> {
    Rc::new(MemoryLenientCredentialNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientCredentialNoticeStore {
    pub inner: Rc<dyn LenientCredentialNoticeStore>,
    pub cache: RefCell<cache::TtlCache<LenientCredentialNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientCredentialNoticeStore for CachedLenientCredentialNoticeStore {
    fn find(&self, code: &str) -> Option<LenientCredentialNotice> {
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

    fn save(&self, item: &LenientCredentialNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientCredentialNotice> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_credential_notice_store(
    inner: Rc<dyn LenientCredentialNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientCredentialNoticeStore> {
    Rc::new(CachedLenientCredentialNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientCredentialNotice>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient credential notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialNoticeQuery {
    pub scope: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientCredentialNoticeQuery {
    pub fn matches(&self, item: &LenientCredentialNotice) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.duration_minutes < self.min_duration_minutes {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LenientCredentialNotice]) -> Vec<LenientCredentialNotice> {
        let matched = collections::filter(items, |candidate: &LenientCredentialNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_credential_notices(
    store: Rc<dyn LenientCredentialNoticeStore>,
    query: &LenientCredentialNoticeQuery,
) -> Vec<LenientCredentialNotice> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn LenientCredentialNoticeStore> = new_memory_lenient_credential_notice_store();
        let item = new_lenient_credential_notice("code-853", "title-699");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
