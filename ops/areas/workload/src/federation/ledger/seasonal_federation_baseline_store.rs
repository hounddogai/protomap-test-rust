use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps seasonal federation baselines by code.
pub trait SeasonalFederationBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<SeasonalFederationBaseline>;
    fn save(&self, item: &SeasonalFederationBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<SeasonalFederationBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemorySeasonalFederationBaselineStore {
    pub items: RefCell<BTreeMap<String, SeasonalFederationBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl SeasonalFederationBaselineStore for MemorySeasonalFederationBaselineStore {
    fn find(&self, code: &str) -> Option<SeasonalFederationBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &SeasonalFederationBaseline) {
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

    fn list(&self) -> Vec<SeasonalFederationBaseline> {
        let mut out: Vec<SeasonalFederationBaseline> = Vec::new();
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

pub fn new_memory_seasonal_federation_baseline_store() -> Rc<dyn SeasonalFederationBaselineStore> {
    Rc::new(MemorySeasonalFederationBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedSeasonalFederationBaselineStore {
    pub inner: Rc<dyn SeasonalFederationBaselineStore>,
    pub cache: RefCell<cache::TtlCache<SeasonalFederationBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl SeasonalFederationBaselineStore for CachedSeasonalFederationBaselineStore {
    fn find(&self, code: &str) -> Option<SeasonalFederationBaseline> {
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

    fn save(&self, item: &SeasonalFederationBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<SeasonalFederationBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_seasonal_federation_baseline_store(
    inner: Rc<dyn SeasonalFederationBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn SeasonalFederationBaselineStore> {
    Rc::new(CachedSeasonalFederationBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<SeasonalFederationBaseline>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects seasonal federation baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFederationBaselineQuery {
    pub label: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl SeasonalFederationBaselineQuery {
    pub fn matches(&self, item: &SeasonalFederationBaseline) -> bool {
        if self.label != "" && (text::normalize(&item.label) != text::normalize(&self.label)) {
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

    pub fn apply(&self, items: &[SeasonalFederationBaseline]) -> Vec<SeasonalFederationBaseline> {
        let matched = collections::filter(items, |candidate: &SeasonalFederationBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_seasonal_federation_baselines(
    store: Rc<dyn SeasonalFederationBaselineStore>,
    query: &SeasonalFederationBaselineQuery,
) -> Vec<SeasonalFederationBaseline> {
    query.apply(&store.list())
}
