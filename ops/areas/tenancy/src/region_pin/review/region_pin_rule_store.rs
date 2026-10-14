use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps region pin rules by code.
pub trait RegionPinRuleStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RegionPinRule>;
    fn save(&self, item: &RegionPinRule);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RegionPinRule>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRegionPinRuleStore {
    pub items: RefCell<BTreeMap<String, RegionPinRule>>,
    pub order: RefCell<Vec<String>>,
}

impl RegionPinRuleStore for MemoryRegionPinRuleStore {
    fn find(&self, code: &str) -> Option<RegionPinRule> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RegionPinRule) {
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

    fn list(&self) -> Vec<RegionPinRule> {
        let mut out: Vec<RegionPinRule> = Vec::new();
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

pub fn new_memory_region_pin_rule_store() -> Rc<dyn RegionPinRuleStore> {
    Rc::new(MemoryRegionPinRuleStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRegionPinRuleStore {
    pub inner: Rc<dyn RegionPinRuleStore>,
    pub cache: RefCell<cache::TtlCache<RegionPinRule>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RegionPinRuleStore for CachedRegionPinRuleStore {
    fn find(&self, code: &str) -> Option<RegionPinRule> {
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

    fn save(&self, item: &RegionPinRule) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RegionPinRule> {
        self.inner.list()
    }
}

pub fn new_cached_region_pin_rule_store(
    inner: Rc<dyn RegionPinRuleStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RegionPinRuleStore> {
    Rc::new(CachedRegionPinRuleStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RegionPinRule>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects region pin rules. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRuleQuery {
    pub scope: String,
    pub min_quota_units: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RegionPinRuleQuery {
    pub fn matches(&self, item: &RegionPinRule) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
            return false;
        }
        if item.quota_units < self.min_quota_units {
            return false;
        }
        if (self.statuses.len() as i64) > 0
            && (!(self.statuses.iter().any(|value| value == &format!("{}", item.status))))
        {
            return false;
        }
        true
    }

    pub fn apply(&self, items: &[RegionPinRule]) -> Vec<RegionPinRule> {
        let matched = collections::filter(items, |candidate: &RegionPinRule| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_region_pin_rules(store: Rc<dyn RegionPinRuleStore>, query: &RegionPinRuleQuery) -> Vec<RegionPinRule> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RegionPinRuleStore> = new_memory_region_pin_rule_store();
        let item = new_region_pin_rule("code-582", "name-723");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
