use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal action item notices by code.
pub trait SeasonalActionItemNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalActionItemNotice>;
    fn save(&self, item: &SeasonalActionItemNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalActionItemNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalActionItemNoticeStore {
    pub items: RefCell<BTreeMap<String, SeasonalActionItemNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalActionItemNoticeStore for MemorySeasonalActionItemNoticeStore {
    fn find(&self, code: &str) -> Option<SeasonalActionItemNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalActionItemNotice) {
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

    fn list(&self) -> Vec<SeasonalActionItemNotice> {
        let mut out: Vec<SeasonalActionItemNotice> = Vec::new();
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

pub fn new_memory_seasonal_action_item_notice_store() -> Rc<dyn SeasonalActionItemNoticeStore> {
    Rc::new(MemorySeasonalActionItemNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalActionItemNoticeStore {
    pub inner: Rc<dyn SeasonalActionItemNoticeStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalActionItemNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalActionItemNoticeStore for CachedSeasonalActionItemNoticeStore {
    fn find(&self, code: &str) -> Option<SeasonalActionItemNotice> {
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

    fn save(&self, item: &SeasonalActionItemNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalActionItemNotice> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_action_item_notice_store(
    inner: Rc<dyn SeasonalActionItemNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalActionItemNoticeStore> {
    Rc::new(CachedSeasonalActionItemNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalActionItemNotice>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal action item notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemNoticeQuery {
    pub title: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalActionItemNoticeQuery {
    pub fn matches(&self, item: &SeasonalActionItemNotice) -> bool {
        if self.title != "" && (text::normalize(&item.title) != text::normalize(&self.title)) {
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

    pub fn apply(&self, items: &[SeasonalActionItemNotice]) -> Vec<SeasonalActionItemNotice> {
        let matched = collections::filter(items, |candidate: &SeasonalActionItemNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_action_item_notices(
    store: Rc<dyn SeasonalActionItemNoticeStore>,
    query: &SeasonalActionItemNoticeQuery,
) -> Vec<SeasonalActionItemNotice> {
    query.apply(&store.list())
}
