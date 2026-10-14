use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps usage overrides by code.
pub trait UsageOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<UsageOverride>;
    fn save(&self, item: &UsageOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<UsageOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryUsageOverrideStore {
    pub items: RefCell<BTreeMap<String, UsageOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl UsageOverrideStore for MemoryUsageOverrideStore {
    fn find(&self, code: &str) -> Option<UsageOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &UsageOverride) {
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

    fn list(&self) -> Vec<UsageOverride> {
        let mut out: Vec<UsageOverride> = Vec::new();
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

pub fn new_memory_usage_override_store() -> Rc<dyn UsageOverrideStore> {
    Rc::new(MemoryUsageOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedUsageOverrideStore {
    pub inner: Rc<dyn UsageOverrideStore>,
    pub cache: RefCell<cache::TtlCache<UsageOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl UsageOverrideStore for CachedUsageOverrideStore {
    fn find(&self, code: &str) -> Option<UsageOverride> {
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

    fn save(&self, item: &UsageOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<UsageOverride> {
        self.inner.list()
    }
}

pub fn new_cached_usage_override_store(
    inner: Rc<dyn UsageOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn UsageOverrideStore> {
    Rc::new(CachedUsageOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<UsageOverride>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects usage overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageOverrideQuery {
    pub category: String,
    pub min_duration_minutes: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl UsageOverrideQuery {
    pub fn matches(&self, item: &UsageOverride) -> bool {
        if self.category != "" && (text::normalize(&item.category) != text::normalize(&self.category)) {
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

    pub fn apply(&self, items: &[UsageOverride]) -> Vec<UsageOverride> {
        let matched = collections::filter(items, |candidate: &UsageOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_usage_overrides(store: Rc<dyn UsageOverrideStore>, query: &UsageOverrideQuery) -> Vec<UsageOverride> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn UsageOverrideStore> = new_memory_usage_override_store();
        let item = new_usage_override("code-358", "label-727");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
