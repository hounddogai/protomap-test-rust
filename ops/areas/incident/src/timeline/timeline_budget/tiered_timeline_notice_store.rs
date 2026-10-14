use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps tiered timeline notices by code.
pub trait TieredTimelineNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<TieredTimelineNotice>;
    fn save(&self, item: &TieredTimelineNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<TieredTimelineNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTieredTimelineNoticeStore {
    pub items: RefCell<BTreeMap<String, TieredTimelineNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl TieredTimelineNoticeStore for MemoryTieredTimelineNoticeStore {
    fn find(&self, code: &str) -> Option<TieredTimelineNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &TieredTimelineNotice) {
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

    fn list(&self) -> Vec<TieredTimelineNotice> {
        let mut out: Vec<TieredTimelineNotice> = Vec::new();
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

pub fn new_memory_tiered_timeline_notice_store() -> Rc<dyn TieredTimelineNoticeStore> {
    Rc::new(MemoryTieredTimelineNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedTieredTimelineNoticeStore {
    pub inner: Rc<dyn TieredTimelineNoticeStore>,
    pub cache: RefCell<cache::TtlCache<TieredTimelineNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl TieredTimelineNoticeStore for CachedTieredTimelineNoticeStore {
    fn find(&self, code: &str) -> Option<TieredTimelineNotice> {
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

    fn save(&self, item: &TieredTimelineNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<TieredTimelineNotice> {
        self.inner.list()
    }
}

pub fn new_cached_tiered_timeline_notice_store(
    inner: Rc<dyn TieredTimelineNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn TieredTimelineNoticeStore> {
    Rc::new(CachedTieredTimelineNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<TieredTimelineNotice>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects tiered timeline notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineNoticeQuery {
    pub source: String,
    pub min_threshold: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl TieredTimelineNoticeQuery {
    pub fn matches(&self, item: &TieredTimelineNotice) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
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

    pub fn apply(&self, items: &[TieredTimelineNotice]) -> Vec<TieredTimelineNotice> {
        let matched = collections::filter(items, |candidate: &TieredTimelineNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_tiered_timeline_notices(
    store: Rc<dyn TieredTimelineNoticeStore>,
    query: &TieredTimelineNoticeQuery,
) -> Vec<TieredTimelineNotice> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn TieredTimelineNoticeStore> = new_memory_tiered_timeline_notice_store();
        let item = new_tiered_timeline_notice("code-269", "label-813");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
