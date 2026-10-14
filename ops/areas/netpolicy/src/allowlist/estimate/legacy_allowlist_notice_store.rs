use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps legacy allowlist notices by code.
pub trait LegacyAllowlistNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LegacyAllowlistNotice>;
    fn save(&self, item: &LegacyAllowlistNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LegacyAllowlistNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLegacyAllowlistNoticeStore {
    pub items: RefCell<BTreeMap<String, LegacyAllowlistNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl LegacyAllowlistNoticeStore for MemoryLegacyAllowlistNoticeStore {
    fn find(&self, code: &str) -> Option<LegacyAllowlistNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LegacyAllowlistNotice) {
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

    fn list(&self) -> Vec<LegacyAllowlistNotice> {
        let mut out: Vec<LegacyAllowlistNotice> = Vec::new();
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

pub fn new_memory_legacy_allowlist_notice_store() -> Rc<dyn LegacyAllowlistNoticeStore> {
    Rc::new(MemoryLegacyAllowlistNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLegacyAllowlistNoticeStore {
    pub inner: Rc<dyn LegacyAllowlistNoticeStore>,
    pub cache: RefCell<cache::TtlCache<LegacyAllowlistNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LegacyAllowlistNoticeStore for CachedLegacyAllowlistNoticeStore {
    fn find(&self, code: &str) -> Option<LegacyAllowlistNotice> {
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

    fn save(&self, item: &LegacyAllowlistNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LegacyAllowlistNotice> {
        self.inner.list()
    }
}

pub fn new_cached_legacy_allowlist_notice_store(
    inner: Rc<dyn LegacyAllowlistNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LegacyAllowlistNoticeStore> {
    Rc::new(CachedLegacyAllowlistNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LegacyAllowlistNotice>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects legacy allowlist notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistNoticeQuery {
    pub title: String,
    pub min_amount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LegacyAllowlistNoticeQuery {
    pub fn matches(&self, item: &LegacyAllowlistNotice) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
            return false;
        }
        if item.amount < self.min_amount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[LegacyAllowlistNotice]) -> Vec<LegacyAllowlistNotice> {
        let matched = collections::filter(items, |candidate: &LegacyAllowlistNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_legacy_allowlist_notices(
    store: Rc<dyn LegacyAllowlistNoticeStore>,
    query: &LegacyAllowlistNoticeQuery,
) -> Vec<LegacyAllowlistNotice> {
    query.apply(&store.list())
}
