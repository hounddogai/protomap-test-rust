use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps rolling nonce baselines by code.
pub trait RollingNonceBaselineStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RollingNonceBaseline>;
    fn save(&self, item: &RollingNonceBaseline);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RollingNonceBaseline>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRollingNonceBaselineStore {
    pub items: RefCell<BTreeMap<String, RollingNonceBaseline>>,
    pub order: RefCell<Vec<String>>,
}

impl RollingNonceBaselineStore for MemoryRollingNonceBaselineStore {
    fn find(&self, code: &str) -> Option<RollingNonceBaseline> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RollingNonceBaseline) {
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

    fn list(&self) -> Vec<RollingNonceBaseline> {
        let mut out: Vec<RollingNonceBaseline> = Vec::new();
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

pub fn new_memory_rolling_nonce_baseline_store() -> Rc<dyn RollingNonceBaselineStore> {
    Rc::new(MemoryRollingNonceBaselineStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRollingNonceBaselineStore {
    pub inner: Rc<dyn RollingNonceBaselineStore>,
    pub cache: RefCell<cache::TtlCache<RollingNonceBaseline>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RollingNonceBaselineStore for CachedRollingNonceBaselineStore {
    fn find(&self, code: &str) -> Option<RollingNonceBaseline> {
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

    fn save(&self, item: &RollingNonceBaseline) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RollingNonceBaseline> {
        self.inner.list()
    }
}

pub fn new_cached_rolling_nonce_baseline_store(
    inner: Rc<dyn RollingNonceBaselineStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RollingNonceBaselineStore> {
    Rc::new(CachedRollingNonceBaselineStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RollingNonceBaseline>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects rolling nonce baselines. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceBaselineQuery {
    pub channel: String,
    pub min_discount: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RollingNonceBaselineQuery {
    pub fn matches(&self, item: &RollingNonceBaseline) -> bool {
        if self.channel != "" && (text::normalize(&item.channel) != text::normalize(&self.channel)) {
            return false;
        }
        if item.discount < self.min_discount {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RollingNonceBaseline]) -> Vec<RollingNonceBaseline> {
        let matched = collections::filter(items, |candidate: &RollingNonceBaseline| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_rolling_nonce_baselines(
    store: Rc<dyn RollingNonceBaselineStore>,
    query: &RollingNonceBaselineQuery,
) -> Vec<RollingNonceBaseline> {
    query.apply(&store.list())
}
