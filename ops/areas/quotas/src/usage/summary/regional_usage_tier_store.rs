use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps regional usage tiers by code.
pub trait RegionalUsageTierStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionalUsageTier>;
    fn save(&self, item: &RegionalUsageTier);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionalUsageTier>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionalUsageTierStore {
    pub items: RefCell<BTreeMap<String, RegionalUsageTier>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionalUsageTierStore for MemoryRegionalUsageTierStore {
    fn find(&self, code: &str) -> Option<RegionalUsageTier> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionalUsageTier) {
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

    fn list(&self) -> Vec<RegionalUsageTier> {
        let mut out: Vec<RegionalUsageTier> = Vec::new();
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

pub fn new_memory_regional_usage_tier_store() -> Rc<dyn RegionalUsageTierStore> {
    Rc::new(MemoryRegionalUsageTierStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionalUsageTierStore {
    pub inner: Rc<dyn RegionalUsageTierStore>,
    pub cache: RefCell<cache::TtlCache<RegionalUsageTier>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionalUsageTierStore for CachedRegionalUsageTierStore {
    fn find(&self, code: &str) -> Option<RegionalUsageTier> {
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

    fn save(&self, item: &RegionalUsageTier) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionalUsageTier> {
        self.inner.list()
    }
}

pub fn new_cached_regional_usage_tier_store(
    inner: Rc<dyn RegionalUsageTierStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionalUsageTierStore> {
    Rc::new(CachedRegionalUsageTierStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionalUsageTier>(1024, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects regional usage tiers. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageTierQuery {
    pub name: String,
    pub min_version: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionalUsageTierQuery {
    pub fn matches(&self, item: &RegionalUsageTier) -> bool {
        if self.name != "" && (text::normalize(&item.name) != text::normalize(&self.name)) {
            return false;
        }
        if item.version < self.min_version {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionalUsageTier]) -> Vec<RegionalUsageTier> {
        let matched = collections::filter(items, |candidate: &RegionalUsageTier| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_regional_usage_tiers(
    store: Rc<dyn RegionalUsageTierStore>,
    query: &RegionalUsageTierQuery,
) -> Vec<RegionalUsageTier> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionalUsageTierStore> = new_memory_regional_usage_tier_store();
        let item = new_regional_usage_tier("code-787", "name-468");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
