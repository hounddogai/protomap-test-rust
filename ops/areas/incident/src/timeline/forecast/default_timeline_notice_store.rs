use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps default timeline notices by code.
pub trait DefaultTimelineNoticeStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<DefaultTimelineNotice>;
    fn save(&self, item: &DefaultTimelineNotice);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<DefaultTimelineNotice>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDefaultTimelineNoticeStore {
    pub items: RefCell<BTreeMap<String, DefaultTimelineNotice>>,
    pub order: RefCell<Vec<String>>,
}

impl DefaultTimelineNoticeStore for MemoryDefaultTimelineNoticeStore {
    fn find(&self, code: &str) -> Option<DefaultTimelineNotice> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &DefaultTimelineNotice) {
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

    fn list(&self) -> Vec<DefaultTimelineNotice> {
        let mut out: Vec<DefaultTimelineNotice> = Vec::new();
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

pub fn new_memory_default_timeline_notice_store() -> Rc<dyn DefaultTimelineNoticeStore> {
    Rc::new(MemoryDefaultTimelineNoticeStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedDefaultTimelineNoticeStore {
    pub inner: Rc<dyn DefaultTimelineNoticeStore>,
    pub cache: RefCell<cache::TtlCache<DefaultTimelineNotice>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl DefaultTimelineNoticeStore for CachedDefaultTimelineNoticeStore {
    fn find(&self, code: &str) -> Option<DefaultTimelineNotice> {
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

    fn save(&self, item: &DefaultTimelineNotice) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<DefaultTimelineNotice> {
        self.inner.list()
    }
}

pub fn new_cached_default_timeline_notice_store(
    inner: Rc<dyn DefaultTimelineNoticeStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn DefaultTimelineNoticeStore> {
    Rc::new(CachedDefaultTimelineNoticeStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<DefaultTimelineNotice>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects default timeline notices. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineNoticeQuery {
    pub country: String,
    pub min_quantity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl DefaultTimelineNoticeQuery {
    pub fn matches(&self, item: &DefaultTimelineNotice) -> bool {
        if self.country != "" && (text::normalize(&item.country) != text::normalize(&self.country)) {
            return false;
        }
        if item.quantity < self.min_quantity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[DefaultTimelineNotice]) -> Vec<DefaultTimelineNotice> {
        let matched = collections::filter(items, |candidate: &DefaultTimelineNotice| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_default_timeline_notices(
    store: Rc<dyn DefaultTimelineNoticeStore>,
    query: &DefaultTimelineNoticeQuery,
) -> Vec<DefaultTimelineNotice> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn DefaultTimelineNoticeStore> = new_memory_default_timeline_notice_store();
        let item = new_default_timeline_notice("code-762", "label-59");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
