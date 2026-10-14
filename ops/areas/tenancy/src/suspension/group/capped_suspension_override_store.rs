use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps capped suspension overrides by code.
pub trait CappedSuspensionOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<CappedSuspensionOverride>;
    fn save(&self, item: &CappedSuspensionOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<CappedSuspensionOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCappedSuspensionOverrideStore {
    pub items: RefCell<BTreeMap<String, CappedSuspensionOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl CappedSuspensionOverrideStore for MemoryCappedSuspensionOverrideStore {
    fn find(&self, code: &str) -> Option<CappedSuspensionOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &CappedSuspensionOverride) {
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

    fn list(&self) -> Vec<CappedSuspensionOverride> {
        let mut out: Vec<CappedSuspensionOverride> = Vec::new();
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

pub fn new_memory_capped_suspension_override_store() -> Rc<dyn CappedSuspensionOverrideStore> {
    Rc::new(MemoryCappedSuspensionOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedCappedSuspensionOverrideStore {
    pub inner: Rc<dyn CappedSuspensionOverrideStore>,
    pub cache: RefCell<cache::TtlCache<CappedSuspensionOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl CappedSuspensionOverrideStore for CachedCappedSuspensionOverrideStore {
    fn find(&self, code: &str) -> Option<CappedSuspensionOverride> {
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

    fn save(&self, item: &CappedSuspensionOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<CappedSuspensionOverride> {
        self.inner.list()
    }
}

pub fn new_cached_capped_suspension_override_store(
    inner: Rc<dyn CappedSuspensionOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn CappedSuspensionOverrideStore> {
    Rc::new(CachedCappedSuspensionOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<CappedSuspensionOverride>(
            128,
            ttl,
            clock.clone(),
        )),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects capped suspension overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionOverrideQuery {
    pub name: String,
    pub min_score: f64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl CappedSuspensionOverrideQuery {
    pub fn matches(&self, item: &CappedSuspensionOverride) -> bool {
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

    pub fn apply(&self, items: &[CappedSuspensionOverride]) -> Vec<CappedSuspensionOverride> {
        let matched = collections::filter(items, |candidate: &CappedSuspensionOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_capped_suspension_overrides(
    store: Rc<dyn CappedSuspensionOverrideStore>,
    query: &CappedSuspensionOverrideQuery,
) -> Vec<CappedSuspensionOverride> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn CappedSuspensionOverrideStore> = new_memory_capped_suspension_override_store();
        let item = new_capped_suspension_override("code-548", "name-545");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
