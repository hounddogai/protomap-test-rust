use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional rule digests by code.
pub trait RegionalRuleDigestStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalRuleDigest>;
    fn save(&self, item: &RegionalRuleDigest);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalRuleDigest>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalRuleDigestStore {
    pub items: RefCell<BTreeMap<String, RegionalRuleDigest>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalRuleDigestStore for MemoryRegionalRuleDigestStore {
    fn find(&self, code: &str) -> Option<RegionalRuleDigest> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalRuleDigest) {
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

    fn list(&self) -> Vec<RegionalRuleDigest> {
        let mut out: Vec<RegionalRuleDigest> = Vec::new();
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

pub fn new_memory_regional_rule_digest_store() -> Rc<dyn RegionalRuleDigestStore> {
    Rc::new(MemoryRegionalRuleDigestStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalRuleDigestStore {
    pub inner: Rc<dyn RegionalRuleDigestStore>,
    pub cache: RefCell<cache::TtlCache<RegionalRuleDigest>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalRuleDigestStore for CachedRegionalRuleDigestStore {
    fn find(&self, code: &str) -> Option<RegionalRuleDigest> {
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

    fn save(&self, item: &RegionalRuleDigest) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalRuleDigest> {
        self.inner.list()
    }
}

pub fn new_cached_regional_rule_digest_store(
    inner: Rc<dyn RegionalRuleDigestStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalRuleDigestStore> {
    Rc::new(CachedRegionalRuleDigestStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalRuleDigest>(512, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional rule digests. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleDigestQuery {
    pub name: String,
    pub min_weight: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalRuleDigestQuery {
    pub fn matches(&self, item: &RegionalRuleDigest) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.weight < self.min_weight {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalRuleDigest]) -> Vec<RegionalRuleDigest> {
        let matched = collections::filter(items, |candidate: &RegionalRuleDigest| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_rule_digests(
    store: Rc<dyn RegionalRuleDigestStore>,
    query: &RegionalRuleDigestQuery,
) -> Vec<RegionalRuleDigest> {
    query.apply(&store.list())
}
