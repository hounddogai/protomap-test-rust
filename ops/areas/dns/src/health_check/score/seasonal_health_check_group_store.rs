use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal health check groups by code.
pub trait SeasonalHealthCheckGroupStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalHealthCheckGroup>;
    fn save(&self, item: &SeasonalHealthCheckGroup);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalHealthCheckGroup>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalHealthCheckGroupStore {
    pub items: RefCell<BTreeMap<String, SeasonalHealthCheckGroup>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalHealthCheckGroupStore for MemorySeasonalHealthCheckGroupStore {
    fn find(&self, code: &str) -> Option<SeasonalHealthCheckGroup> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalHealthCheckGroup) {
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

    fn list(&self) -> Vec<SeasonalHealthCheckGroup> {
        let mut out: Vec<SeasonalHealthCheckGroup> = Vec::new();
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

pub fn new_memory_seasonal_health_check_group_store() -> Rc<dyn SeasonalHealthCheckGroupStore> {
    Rc::new(MemorySeasonalHealthCheckGroupStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalHealthCheckGroupStore {
    pub inner: Rc<dyn SeasonalHealthCheckGroupStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalHealthCheckGroup>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalHealthCheckGroupStore for CachedSeasonalHealthCheckGroupStore {
    fn find(&self, code: &str) -> Option<SeasonalHealthCheckGroup> {
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

    fn save(&self, item: &SeasonalHealthCheckGroup) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalHealthCheckGroup> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_health_check_group_store(
    inner: Rc<dyn SeasonalHealthCheckGroupStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalHealthCheckGroupStore> {
    Rc::new(CachedSeasonalHealthCheckGroupStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalHealthCheckGroup>(
            256,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal health check groups. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckGroupQuery {
    pub source: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalHealthCheckGroupQuery {
    pub fn matches(&self, item: &SeasonalHealthCheckGroup) -> bool {
        if self.source != "" && (text::normalize(&item.source) != text::normalize(&self.source)) {
            return false;
        }
        if item.capacity < self.min_capacity {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[SeasonalHealthCheckGroup]) -> Vec<SeasonalHealthCheckGroup> {
        let matched = collections::filter(items, |candidate: &SeasonalHealthCheckGroup| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_health_check_groups(
    store: Rc<dyn SeasonalHealthCheckGroupStore>,
    query: &SeasonalHealthCheckGroupQuery,
) -> Vec<SeasonalHealthCheckGroup> {
    query.apply(&store.list())
}
