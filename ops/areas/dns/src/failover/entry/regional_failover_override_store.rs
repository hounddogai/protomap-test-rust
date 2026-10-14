use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional failover overrides by code.
pub trait RegionalFailoverOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalFailoverOverride>;
    fn save(&self, item: &RegionalFailoverOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalFailoverOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalFailoverOverrideStore {
    pub items: RefCell<BTreeMap<String, RegionalFailoverOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalFailoverOverrideStore for MemoryRegionalFailoverOverrideStore {
    fn find(&self, code: &str) -> Option<RegionalFailoverOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalFailoverOverride) {
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

    fn list(&self) -> Vec<RegionalFailoverOverride> {
        let mut out: Vec<RegionalFailoverOverride> = Vec::new();
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

pub fn new_memory_regional_failover_override_store() -> Rc<dyn RegionalFailoverOverrideStore> {
    Rc::new(MemoryRegionalFailoverOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalFailoverOverrideStore {
    pub inner: Rc<dyn RegionalFailoverOverrideStore>,
    pub cache: RefCell<cache::TtlCache<RegionalFailoverOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalFailoverOverrideStore for CachedRegionalFailoverOverrideStore {
    fn find(&self, code: &str) -> Option<RegionalFailoverOverride> {
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

    fn save(&self, item: &RegionalFailoverOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalFailoverOverride> {
        self.inner.list()
    }
}

pub fn new_cached_regional_failover_override_store(
    inner: Rc<dyn RegionalFailoverOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalFailoverOverrideStore> {
    Rc::new(CachedRegionalFailoverOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalFailoverOverride>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional failover overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverOverrideQuery {
    pub locale: String,
    pub min_minimum: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalFailoverOverrideQuery {
    pub fn matches(&self, item: &RegionalFailoverOverride) -> bool {
        if self.locale != "" && (text::normalize(&item.locale) != text::normalize(&self.locale)) {
            return false;
        }
        if item.minimum < self.min_minimum {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalFailoverOverride]) -> Vec<RegionalFailoverOverride> {
        let matched = collections::filter(items, |candidate: &RegionalFailoverOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_failover_overrides(
    store: Rc<dyn RegionalFailoverOverrideStore>,
    query: &RegionalFailoverOverrideQuery,
) -> Vec<RegionalFailoverOverride> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalFailoverOverrideStore> = new_memory_regional_failover_override_store();
        let item = new_regional_failover_override("code-59", "title-785");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
