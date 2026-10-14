use super::*;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::cache;
use acme_kit::clock;
use acme_kit::collections;
use acme_kit::text;

/// Keeps runbook overrides by code.
pub trait RunbookOverrideStore: std::fmt::Debug {
    fn find(&self, code: &str) -> Option<RunbookOverride>;
    fn save(&self, item: &RunbookOverride);
    fn remove(&self, code: &str) -> bool;
    fn list(&self) -> Vec<RunbookOverride>;
}

/// Keeps records in memory, in the order they were first saved.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRunbookOverrideStore {
    pub items: RefCell<BTreeMap<String, RunbookOverride>>,
    pub order: RefCell<Vec<String>>,
}

impl RunbookOverrideStore for MemoryRunbookOverrideStore {
    fn find(&self, code: &str) -> Option<RunbookOverride> {
        let found = self.items.borrow().get(code).cloned();
        let Some(found) = found else {
            return None;
        };
        Some(found)
    }

    fn save(&self, item: &RunbookOverride) {
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

    fn list(&self) -> Vec<RunbookOverride> {
        let mut out: Vec<RunbookOverride> = Vec::new();
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

pub fn new_memory_runbook_override_store() -> Rc<dyn RunbookOverrideStore> {
    Rc::new(MemoryRunbookOverrideStore {
        items: RefCell::new(BTreeMap::new()),
        order: RefCell::new(Vec::new()),
    })
}

/// Serves reads from a short-lived cache in front of another store.
#[derive(Debug, Clone)]
pub struct CachedRunbookOverrideStore {
    pub inner: Rc<dyn RunbookOverrideStore>,
    pub cache: RefCell<cache::TtlCache<RunbookOverride>>,
    pub hits: Cell<i64>,
    pub misses: Cell<i64>,
}

impl RunbookOverrideStore for CachedRunbookOverrideStore {
    fn find(&self, code: &str) -> Option<RunbookOverride> {
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

    fn save(&self, item: &RunbookOverride) {
        self.inner.save(item);
        self.cache.borrow_mut().put(&item.code, item);
    }

    fn remove(&self, code: &str) -> bool {
        // A cached copy expires on its own; reads after a removal may still see it until then.
        self.inner.remove(code)
    }

    fn list(&self) -> Vec<RunbookOverride> {
        self.inner.list()
    }
}

pub fn new_cached_runbook_override_store(
    inner: Rc<dyn RunbookOverrideStore>,
    ttl: i64,
    clock: Rc<dyn clock::Clock>,
) -> Rc<dyn RunbookOverrideStore> {
    Rc::new(CachedRunbookOverrideStore {
        inner: inner.clone(),
        cache: RefCell::new(cache::new_ttl_cache::<RunbookOverride>(256, ttl, clock.clone())),
        hits: Cell::new(0),
        misses: Cell::new(0),
    })
}

/// Selects runbook overrides. Empty fields match all.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookOverrideQuery {
    pub scope: String,
    pub min_capacity: i64,
    pub statuses: Vec<String>,
    pub limit: i64,
}

impl RunbookOverrideQuery {
    pub fn matches(&self, item: &RunbookOverride) -> bool {
        if self.scope != "" && (text::normalize(&item.scope) != text::normalize(&self.scope)) {
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

    pub fn apply(&self, items: &[RunbookOverride]) -> Vec<RunbookOverride> {
        let matched = collections::filter(items, |candidate: &RunbookOverride| self.matches(candidate));
        if self.limit > 0 && ((matched.len() as i64) > self.limit) {
            return collections::take(&matched, self.limit);
        }
        matched
    }
}

pub fn find_runbook_overrides(
    store: Rc<dyn RunbookOverrideStore>,
    query: &RunbookOverrideQuery,
) -> Vec<RunbookOverride> {
    query.apply(&store.list())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_what_it_saved() {
        let store: Rc<dyn RunbookOverrideStore> = new_memory_runbook_override_store();
        let item = new_runbook_override("code-2", "name-323");
        store.save(&item);
        let Some(found) = store.find(&item.code) else {
            panic!("unexpected empty result");
        };
        assert_eq!(found.code, item.code, "code");
        assert_eq!(store.list().len() as i64, 1, "count");
    }
}
