use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps page schedules by code.
pub trait PageScheduleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<PageSchedule>;
    fn save(&self, item: &PageSchedule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<PageSchedule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryPageScheduleStore {
    pub items: RefCell<BTreeMap<String, PageSchedule>>,
    pub order: RefCell<Vec<String>>,
}

impl PageScheduleStore for MemoryPageScheduleStore {
    fn find(&self, code: &str) -> Option<PageSchedule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &PageSchedule) {
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

    fn list(&self) -> Vec<PageSchedule> {
        let mut out: Vec<PageSchedule> = Vec::new();
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

pub fn new_memory_page_schedule_store() -> Rc<dyn PageScheduleStore> {
    Rc::new(MemoryPageScheduleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedPageScheduleStore {
    pub inner: Rc<dyn PageScheduleStore>,
    pub cache: RefCell<cache::TtlCache<PageSchedule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl PageScheduleStore for CachedPageScheduleStore {
    fn find(&self, code: &str) -> Option<PageSchedule> {
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

    fn save(&self, item: &PageSchedule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<PageSchedule> {
        self.inner.list()
    }
}

pub fn new_cached_page_schedule_store(
    inner: Rc<dyn PageScheduleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn PageScheduleStore> {
    Rc::new(CachedPageScheduleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<PageSchedule>(128, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects page schedules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct PageScheduleQuery {
    pub name: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl PageScheduleQuery {
    pub fn matches(&self, item: &PageSchedule) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.score < self.min_score {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[PageSchedule]) -> Vec<PageSchedule> {
        let matched = collections::filter(items, |candidate: &PageSchedule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_page_schedules(store: Rc<dyn PageScheduleStore>, query: &PageScheduleQuery) -> Vec<PageSchedule> {
    query.apply(&store.list())
}
