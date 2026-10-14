use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps lenient rule notices by code.
pub trait LenientRuleNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<LenientRuleNotice>;
    fn save(&self, item: &LenientRuleNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<LenientRuleNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLenientRuleNoticeStore {
    pub items: RefCell<BTreeMap<String, LenientRuleNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl LenientRuleNoticeStore for MemoryLenientRuleNoticeStore {
    fn find(&self, code: &str) -> Option<LenientRuleNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &LenientRuleNotice) {
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

    fn list(&self) -> Vec<LenientRuleNotice> {
        let mut out: Vec<LenientRuleNotice> = Vec::new();
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

pub fn new_memory_lenient_rule_notice_store() -> Rc<dyn LenientRuleNoticeStore> {
    Rc::new(MemoryLenientRuleNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedLenientRuleNoticeStore {
    pub inner: Rc<dyn LenientRuleNoticeStore>,
    pub cache: RefCell<cache::TtlCache<LenientRuleNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl LenientRuleNoticeStore for CachedLenientRuleNoticeStore {
    fn find(&self, code: &str) -> Option<LenientRuleNotice> {
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

    fn save(&self, item: &LenientRuleNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<LenientRuleNotice> {
        self.inner.list()
    }
}

pub fn new_cached_lenient_rule_notice_store(
    inner: Rc<dyn LenientRuleNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn LenientRuleNoticeStore> {
    Rc::new(CachedLenientRuleNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<LenientRuleNotice>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects lenient rule notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleNoticeQuery {
    pub title: String,
    pub min_limit: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl LenientRuleNoticeQuery {
    pub fn matches(&self, item: &LenientRuleNotice) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[LenientRuleNotice]) -> Vec<LenientRuleNotice> {
        let matched = collections::filter(items, |candidate: &LenientRuleNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_lenient_rule_notices(
    store: Rc<dyn LenientRuleNoticeStore>,
    query: &LenientRuleNoticeQuery,
) -> Vec<LenientRuleNotice> {
    query.apply(&store.list())
}
